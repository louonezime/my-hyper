use std::collections::HashMap;
use toml::Value;

use http::Uri;
use hyper::header::HeaderMap;

use super::config::setup_servers;

pub fn print_formatted_headers(headers: &HeaderMap) {
    for (key, value) in headers.iter() {
        match value.to_str() {
            Ok(val) => println!("\t{}:\t{}", key, val),
            Err(err) => eprintln!("Error: {}", err),
        };
    }
    println!();
}

pub fn clean_url(mut uri: &str) -> String {
    if uri.ends_with('/') {
        uri = &uri[..uri.len() - 1];
    }
    uri.to_string()
}

pub fn find_resources_refs(
    referer: &str,
    subpath: &str,
    servers: HashMap<String, String>,
) -> Option<String> {
    servers.get(referer).map(|ref_host| {
        let target_url = if ref_host.ends_with('/') {
            let (tmp, _) = ref_host.split_at(ref_host.len() - 1);
            format!("{}{}", tmp, subpath)
        } else {
            format!("{}{}", ref_host, subpath)
        };
        target_url
    })
}

pub fn determine_target(path_ref: &str, req_uri: &hyper::Uri, config: Value) -> Result<String, ()> {
    let servers = match setup_servers(config) {
        Some(res) => res,
        None => return Err(()),
    };

    Ok(match servers.get(req_uri.path()) {
        Some(proxy_path) => proxy_path.to_string(),
        None => {
            let ref_uri = match path_ref.parse::<Uri>() {
                Ok(value) => value,
                Err(_) => return Err(()),
            };
            find_resources_refs(ref_uri.path(), req_uri.path(), servers).ok_or(())?
        }
    })
}

pub fn _strncmp(s1: &str, s2: &str, n: usize) -> bool {
    let mut iter1 = s1.chars();
    let mut iter2 = s2.chars();

    for _ in 0..n {
        match (iter1.next(), iter2.next()) {
            (Some(c1), Some(c2)) if c1 != c2 => return false,
            (None, None) => return true,
            (None, Some(_)) | (Some(_), None) => return false,
            _ => {}
        }
    }

    true
}

pub fn common_prefix(s1: &str, s2: &str) -> String {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .map(|(c, _)| c)
        .collect()
}
