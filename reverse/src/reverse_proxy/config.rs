use std::fs::File;
use std::io::{prelude::Read, BufReader};

use std::collections::HashMap;
use std::string::String;
use toml::Value;

use std::error::Error;

use super::basic::ServerCredentials;
use super::utils::clean_url;

pub fn setup_basic(config: Value) -> Option<HashMap<String, ServerCredentials>> {
    let mut map = HashMap::new();
    let basic = match config["basic"].as_table() {
        Some(basic) => basic,
        None => {
            eprintln!("Error parsing the [basic] structure");
            return None;
        }
    };

    for (path, auth_info) in basic {
        let realm = auth_info["realm"].as_str()?;
        let username = auth_info["username"].as_str()?;
        let password = auth_info["password"].as_str()?;

        let credential = ServerCredentials::new(realm, username, password);
        map.insert(path.to_string(), credential);
    }

    Some(map)
}

pub fn setup_form(config: Value) -> Option<HashMap<String, Vec<(String, String)>>> {
    let mut map = HashMap::new();

    match config["form"].as_table() {
        Some(form_table) => {
            for (path, form_info) in form_table {
                if let Some(form_info_table) = form_info.as_table() {
                    let mut inner_vec = Vec::new();

                    for (key, value) in form_info_table {
                        if let Some(inner_value) = value.as_str() {
                            inner_vec.push((key.to_string(), inner_value.to_string()));
                        } else {
                            eprintln!("Error parsing {} for path: {}", key, path);
                            continue;
                        }
                    }
                    let path = &clean_url(path);
                    map.insert(path.clone(), inner_vec);
                }
            }
        }
        None => {
            eprintln!("Error parsing the [form] structure");
            return None;
        }
    };

    match map.is_empty() {
        true => {
            eprintln!("Form authentication table isn't properly configured");
            None
        }
        false => Some(map),
    }
}

pub fn setup_servers(config: Value) -> Option<HashMap<String, String>> {
    let redirections = match config["redirections"].as_table() {
        Some(servers) => servers,
        None => {
            eprintln!("Error parsing the [redirections] structure");
            return None;
        }
    };

    let map: HashMap<String, String> = redirections
        .iter()
        .map(|(k, v)| {
            let value_as_string = match v.as_str() {
                Some(s) => s.to_string(),
                None => {
                    eprintln!("Error converting value {:?} to key {:?}", v, k);
                    return (k.clone(), "".to_string());
                }
            };
            (k.clone(), value_as_string)
        })
        .collect();

    Some(map)
}

pub fn define_conf(filename: &str) -> Result<Value, Box<dyn Error>> {
    let toml_script = match File::open(filename) {
        Ok(file) => {
            let mut contents = String::new();
            let mut buf_reader = BufReader::new(file);
            if let Err(err) = buf_reader.read_to_string(&mut contents) {
                eprintln!("Error reading the {} file : {}", filename, err);
                return Err(err.into());
            }
            contents
        }
        Err(err) => {
            eprint!("Error opening the {} file : {}", filename, err);
            return Err(err.into());
        }
    };

    let parsed_toml: Value = match toml::from_str(&toml_script) {
        Ok(script) => script,
        Err(err) => {
            eprintln!("Failed to parse TOML [config] script => Error: {}", err);
            return Err(err.into());
        }
    };

    Ok(parsed_toml)
}
