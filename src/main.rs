extern crate chrono;
extern crate dayu;
extern crate libxml;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate url;

mod cache;
mod config;

use cache::Cache;
use chrono::{Local, NaiveDate, Duration};
use config::Config;
use dayu::Dayu;
use libxml::parser::Parser;
use libxml::tree::Node;
use regex::Regex;
use serde_json::Value;
use url::Url;

#[derive(Debug)]
struct Convention {
    name: String,
    time: (NaiveDate, NaiveDate),
    area: String,
}

impl Convention {
    fn new() -> Convention {
        Convention {
            name: String::new(),
            time: (
                NaiveDate::from_ymd(2018, 1, 1),
                NaiveDate::from_ymd(2018, 1, 1),
            ),
            area: String::new(),
        }
    }
}

impl<'a> From<&'a Convention> for Value {
    fn from(convention: &'a Convention) -> Value {
        json!({
            "name": convention.name,
            "area": convention.area,
            "start": convention.time.0.format("%Y-%m-%d").to_string(),
            "end": convention.time.1.format("%Y-%m-%d").to_string(),
        })
    }
}

fn truncate_content(content: String) -> String {
    let mut new_chars = content.trim().chars().take(20).collect::<Vec<char>>();

    if new_chars.len() == 20 {
        new_chars.pop();
        new_chars.push('…');
    }

    new_chars.into_iter().collect()
}

fn find_conventions(root: Node, today: &NaiveDate, conventions: &mut Vec<Convention>) -> bool {
    if root.get_attribute("class").as_ref().map(|v| v.as_str()) == Some("events-info-list") {
        let mut index = 0;
        let mut convention = Convention::new();
        let parent = root.get_parent().unwrap();

        for node in parent.get_child_nodes() {
            if node.get_name().as_str() == "h3" {
                convention.name = truncate_content(node.get_content());
            }
        }

        for node in root.get_child_nodes() {
            if node.get_name().as_str() == "li" {
                let content = node.get_content()
                    .chars()
                    .skip_while(|c| *c != '：')
                    .skip(1)
                    .collect::<String>()
                    .trim_right_matches('\u{200b}')
                    .to_string();

                match index {
                    0 => {
                        let regex = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();

                        for (index, capture) in regex.captures_iter(&content).enumerate() {
                            let date = NaiveDate::parse_from_str(&capture[0], "%Y-%m-%d").unwrap();
                            match index {
                                0 => convention.time.0 = date,
                                1 => convention.time.1 = date,
                                _ => (),
                            }
                        }
                    }
                    1 => convention.area = truncate_content(content),
                    _ => (),
                }
                index += 1;
            }
        }

        if convention.time.1 >= *today {
            conventions.push(convention)
        } else {
            return false;
        }
    }

    for child_node in root.get_child_nodes() {
        if !find_conventions(child_node, today, conventions) {
            return false;
        }
    }

    true
}

fn main() {
    let mut conventions = Vec::new();
    let today = Local::today().naive_local();
    let mut url =
        Url::parse("http://www.neccsh.com/cecsh/exhibitioninfo/exhibitionlist.jspx").unwrap();

    println!("start conventions check.");

    for i in 1..6 {
        url.set_query(Some(&format!("pageNo={}", i)));
        let html = reqwest::get(url.clone()).unwrap().text().unwrap();
        let root = Parser::default().parse_string(&html).unwrap();

        if !find_conventions(root.as_node(), &today, &mut conventions) {
            break;
        }
    }

    if conventions.is_empty() {
        panic!("conventions is empty.");
    } else {
        conventions.reverse();
        let cache = Cache::load();
        for convention in conventions {
            if today >= convention.time.0 - Duration::days(1) || convention.time.1 <= today {
                if today > cache {
                    let config = Config::load();
                    let mut dayu = Dayu::new();
                    let value = Value::from(&convention);

                    println!(
                        "convention: {}",
                        serde_json::to_string_pretty(&value).unwrap()
                    );

                    dayu.set_access_key(&config.access_key);
                    dayu.set_access_secret(&config.access_secret);
                    dayu.set_sign_name(&config.sign_name);
                    dayu.sms_send(&config.phones[..], "SMS_133960571", Some(&value))
                        .unwrap();

                    Cache::save(convention.time.1);
                } else {
                    println!("has sended.");
                }
                break;
            }
        }
    }
}
