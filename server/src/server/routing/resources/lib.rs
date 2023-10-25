use std::sync::{Arc, Mutex};
use std::collections::HashSet;

pub fn print_resources(resources: Arc<Mutex<HashSet<String>>>) {
    let resources_set = resources.lock().unwrap();

    println!("List of Resources:");
    for source in resources_set.iter() {
        println!("**{}", source);
    }
}

pub fn empty_resources(resources: &Arc<Mutex<HashSet<String>>>) {
    let mut resources_set = resources.lock().unwrap();
    resources_set.clear();
}

pub fn find_resource<'a> (
    resources: &'a Arc<Mutex<HashSet<String>>>, refer: &'a str
) -> Option<String> {
    let resources_set = resources.lock().unwrap();

    for source in resources_set.clone().into_iter() {
        if source.starts_with(refer) {
            return Some(source);
        }
    }
    return None
}

pub fn is_resources_empty(resources: &Arc<Mutex<HashSet<String>>>) -> bool {
    let resources_set = resources.lock().unwrap();
    resources_set.is_empty()
}
