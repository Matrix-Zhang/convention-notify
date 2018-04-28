use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};

use chrono::NaiveDate;

pub struct Cache;

impl Cache {
    pub fn load() -> NaiveDate {
        File::open(".cache")
            .and_then(|mut file| {
                let mut cache = String::new();
                file.read_to_string(&mut cache).map(|_| cache)
            })
            .and_then(|ref cache| {
                NaiveDate::parse_from_str(cache, "%Y-%m-%d")
                    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
            })
            .unwrap_or_else(|_| NaiveDate::from_ymd(2018, 1, 1))
    }

    pub fn save(date: NaiveDate) {
        File::create(".cache")
            .and_then(|mut file| file.write_all(date.format("%Y-%m-%d").to_string().as_bytes()))
            .unwrap();
    }
}
