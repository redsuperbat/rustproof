use dirs::{config_dir, data_dir};
use log::info;
use reqwest::get;
use serde;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Deserialize, Clone)]
pub struct Dictionary {
    pub language: String,
    pub aff: String,
    pub dic: String,
}

#[derive(Clone)]
pub struct DictionaryPath {
    pub aff: PathBuf,
    pub dic: PathBuf,
}

impl Dictionary {
    pub async fn resolve(&self) -> DictionaryPath {
        let data_dir = data_dir()
            .expect("Could not find data dir")
            .join("rustproof")
            .join(&self.language);
        ensure_directory(&data_dir).await;
        let aff_path = data_dir.join("index.aff");
        let dic_path = data_dir.join("index.dic");
        let aff = self.aff.clone();
        let dic = self.dic.clone();
        Dictionary::download_if_not_exists(&aff_path, &aff).await;
        Dictionary::download_if_not_exists(&dic_path, &dic).await;
        DictionaryPath {
            aff: aff_path,
            dic: dic_path,
        }
    }

    async fn download_if_not_exists(buf: &PathBuf, url: &str) {
        if buf.exists() {
            return;
        };
        info!("{:?}", buf);
        let mut file = fs::File::create(&buf).await.expect("Could not create file");
        let response = get(url).await.expect("Unable to send request");
        let bytes = response.bytes().await.expect("Unable to get bytes");
        file.write_all(&bytes).await.expect("Unable to write file");
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_dict_path")]
    pub dict_path: PathBuf,
    #[serde(default = "default_dictionaries")]
    pub dictionaries: Vec<Dictionary>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dict_path: default_dict_path(),
            dictionaries: default_dictionaries(),
        }
    }
}

fn default_dictionaries() -> Vec<Dictionary> {
    let en_au_coding = Dictionary {
        language: "en_AU_coding".to_string(),
        aff:"https://raw.githubusercontent.com/maxmilton/hunspell-dictionary/refs/heads/master/en_AU.aff".to_string() ,
        dic: "https://raw.githubusercontent.com/maxmilton/hunspell-dictionary/refs/heads/master/en_AU.dic".to_string()
    };
    let en_us = Dictionary {
           language: "en_US".to_string(),
           aff:"https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.aff".to_string() ,
           dic: "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.dic".to_string()
       };
    vec![en_au_coding, en_us]
}

async fn ensure_directory(path: &PathBuf) {
    if path.exists() {
        return;
    }
    fs::create_dir_all(path)
        .await
        .expect("Could not create directories")
}

fn default_dict_path() -> PathBuf {
    let mut path = config_dir().expect("Unable to get config dir");
    path.push("rustproof");
    path.push("dict.txt");
    path
}

pub fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().map(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}
