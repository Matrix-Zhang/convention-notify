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
        File::open("config.toml")
            .and_then(|mut file| {
                let mut config = String::new();
                file.read_to_string(&mut config)
                    .map(|_| toml::from_str(&config).expect("toml load fail."))
            })
            .expect("open config.toml failed.")
    }
}
