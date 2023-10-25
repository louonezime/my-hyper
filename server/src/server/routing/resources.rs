use scraper::{Html, Selector};

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

mod scripts;
pub mod lib;

fn check_script(document: Html) -> Vec<Option<String>> {
    let mut collect_script_ref = Vec::new();
    let script_parser = Selector::parse("script").unwrap();

    for element in document.select(&script_parser) {
        let text = element.text().collect::<String>();
        let lines: Vec<&str> = text.split('\n').collect();
        for line in &lines {
            if let Some(res_link_ref) = scripts::inspect_fetch(line) {
                collect_script_ref.push(Some(res_link_ref));
            }
        }
    }
    collect_script_ref
}

pub fn check_resources(
    full_body_str: &String, resources: &Arc<Mutex<HashSet<String>>>) {
    let document = Html::parse_document(&full_body_str);

    let src_parser = Selector::parse("[src]").unwrap();
    let href_parser = Selector::parse("[href]").unwrap();

    let mut resources_set = resources.lock().unwrap();
    for element in document.select(&src_parser) {
        if let Some(src) = element.value().attr("src") {
            resources_set.insert(src.to_string());
        }
    }

    for element in document.select(&href_parser) {
        if let Some(href) = element.value().attr("href") {
            resources_set.insert(href.to_string());
        }
    }
    let script = check_script(document.clone());
    for result in script {
        if let Some(script_elem) = result {
            resources_set.insert(script_elem.to_string());
        }
    }
}
