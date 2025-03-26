use dirs::config_dir;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub dict_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dict_path: Config::default_dict_path(),
        }
    }
}

impl Config {
    fn default_dict_path() -> PathBuf {
        let mut path = config_dir().expect("Unable to get config dir");
        path.push("rustproof");
        path.push("dict.txt");
        path
    }
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
