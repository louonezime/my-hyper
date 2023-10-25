use std::collections::HashMap;
use std::sync::Mutex;

use http::Uri;

use once_cell::sync::Lazy;

type GlobalHashMap = HashMap<String, String>;

static GLOBAL_MAP: Lazy<Mutex<GlobalHashMap>> = Lazy::new(|| {
    let mut map = GlobalHashMap::new();

    map.insert("/bastion".to_string(), "https://10.10.176.126/".to_string());
    map.insert("/app2".to_string(), "https://demoqa.com/".to_string());
    map.insert("/git".to_string(), "https://github.com/".to_string());
    map.insert("/simple".to_string(), "http://www.columbia.edu/~fdc/sample.html".to_string());
    map.insert("/def".to_string(), "https://httpbin.org/".to_string());
    map.insert("/auth".to_string(), "https://httpbin.org/basic-auth/foo/bar".to_string());
    map.insert("/art".to_string(), "https://www.luxinyaoportfolio.com/".to_string());
    map.insert("/rand".to_string(), "https://www.ryanhaskins.com/".to_string());
    Mutex::new(map)
});

pub fn _print_global_map() {
    println!("GLOBAL MAP:");
    let global_map = GLOBAL_MAP.lock().unwrap();

    for (key, value) in global_map.iter() {
        println!("Key: {}", key);
        println!("Value: {}", value);
        println!("------------------");
    }
}

pub fn match_servers(path: &str) -> Option<String> {
    let map = GLOBAL_MAP.lock().unwrap();

    for (key, value) in map.iter() {
        if key == path {
            return Some(value.to_string());
        }
    }
    None
}


pub fn find_resources_refs(referer: &str, subpath: &str) -> Option<String> {
    match_servers(&referer).map(|ref_host| {
        let target_url = if ref_host.ends_with('/') {
            let (tmp, _) = ref_host.split_at(ref_host.len() - 1);
            format!("{}{}", tmp, subpath)
        } else {
            format!("{}{}", ref_host, subpath)
        };
        target_url
    })
}

pub fn determine_target(path_ref: String, req_uri: &hyper::Uri) -> Result<String, ()> {

    if req_uri.path().starts_with("/") {
        Ok(match match_servers(req_uri.path()) {
            Some(proxy_path) => proxy_path,
            _ => {
                let ref_uri = path_ref.parse::<Uri>().unwrap();
                find_resources_refs(ref_uri.path(), req_uri.path()).ok_or(())?
            }
        })
    } else {
        Err(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_match_servers() {
        let test_cases = vec![
            ("/bastion", vec!["https://10.10.176.126/"]),
            ("/app2", vec!["https://demoqa.com/"]),
            ("/git", vec!["https://github.com/"]),
            ("/simple", vec!["http://www.columbia.edu/~fdc/sample.html"]),
            ("/def", vec!["https://httpbin.org/"]),
            ("/auth", vec!["https://httpbin.org/basic-auth/foo/bar"]),
            ("/art", vec!["https://www.luxinyaoportfolio.com/"]),
            ("/rand", vec!["https://www.ryanhaskins.com/"]),
            ("/nonexistent", vec![]),
        ];

        for (path, expected) in test_cases {
            let result = match_servers(path);
            assert_eq!(result, expected.iter().next().map(|&x| x.to_string()));
        }
    }
}
