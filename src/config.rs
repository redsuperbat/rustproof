use dirs::{config_dir, data_dir};
use log::info;
use reqwest::get;
use serde;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tower_lsp::lsp_types::DiagnosticSeverity;

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
    pub fn new(language: &str, aff: &str, dic: &str) -> Self {
        Self {
            language: language.to_string(),
            aff: aff.to_string(),
            dic: dic.to_string(),
        }
    }
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
#[serde(rename_all = "lowercase")] // Ensures case-insensitivity
pub enum ConfigDiagnosticSeverity {
    Error,
    Info,
    Warning,
    Hint,
}

impl ConfigDiagnosticSeverity {
    pub fn to_lsp_diagnostic(&self) -> DiagnosticSeverity {
        match self {
            ConfigDiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
            ConfigDiagnosticSeverity::Info => DiagnosticSeverity::INFORMATION,
            ConfigDiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
            ConfigDiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_dict_path")]
    pub dict_path: PathBuf,
    #[serde(default = "default_dictionaries")]
    pub dictionaries: Vec<Dictionary>,
    #[serde(default = "default_diagnostic_severity")]
    pub diagnostic_severity: ConfigDiagnosticSeverity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dict_path: default_dict_path(),
            dictionaries: default_dictionaries(),
            diagnostic_severity: default_diagnostic_severity(),
        }
    }
}

fn default_diagnostic_severity() -> ConfigDiagnosticSeverity {
    ConfigDiagnosticSeverity::Error
}

fn default_dictionaries() -> Vec<Dictionary> {
    let base_url =
        "https://raw.githubusercontent.com/redsuperbat/rustproof/refs/heads/main/dictionaries";
    vec![
        Dictionary::new(
            "en-code",
            &(base_url.to_string() + "/en-code/index.aff"),
            &(base_url.to_string() + "/en-code/index.aff"),
        ),
        Dictionary::new(
            "en",
            &(base_url.to_string() + "/en/index.aff"),
            &(base_url.to_string() + "/en/index.aff"),
        ),
    ]
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
