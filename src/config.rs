use dirs::config_dir;
use std::path::PathBuf;

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
