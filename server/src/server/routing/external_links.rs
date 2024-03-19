use regex::Regex;

use std::collections::HashSet;

fn ref_parsing(hreference_list: HashSet<String>, path: &str) {
    for href in hreference_list {
        print!("{}, ", href);
    }
}

pub fn check_href_links(body: String, proxy_path: &str) -> HashSet<String> {
    let mut collected_links: HashSet<String> = HashSet::new();
    let body_line: Vec<&str> = body.split(" \n").collect();

    for lines in body_line {
        let reg = Regex::new(r#"<a[^>]*href=["']([^"']+)["'][^>]*>"#).unwrap();

        for capt in reg.captures_iter(lines) {
            if let Some(href) = capt.get(1) {
                println!("href: {}", href.as_str());
                if !href.as_str().starts_with("/") {
                    collected_links.insert(href.as_str().to_string());
                }
            }
        }
        ref_parsing(collected_links.clone(), proxy_path);
    }
    print!("\n");
    collected_links
}
