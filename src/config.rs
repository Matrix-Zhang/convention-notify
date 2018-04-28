extern crate toml;

use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct Config {
    pub access_key: String,
    pub access_secret: String,
    pub sign_name: String,
    pub phones: Vec<String>,
}

impl Config {
    pub fn load() -> Config {
        let mut file = File::open("config.toml").unwrap();
        let mut config = String::new();
        file.read_to_string(&mut config).unwrap();
        toml::from_str(&config).unwrap()
    }
}
