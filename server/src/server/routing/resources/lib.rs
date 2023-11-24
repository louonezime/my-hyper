use std::sync::{Arc, Mutex};
use std::collections::HashSet;


/// Prints the request target's list of resources
///
/// * `resources` - A Mutex HashSet of Strings
///
pub fn print_resources(resources: Arc<Mutex<HashSet<String>>>) {
    let resources_set = resources.lock().unwrap();

    println!("List of Resources:");
    for source in resources_set.iter() {
        println!("**{}", source);
    }
}

/// Empties the request target's list of resources
///
/// * `resources` - Address of Mutex HashSet of Strings
///
pub fn empty_resources(resources: &Arc<Mutex<HashSet<String>>>) {
    let mut resources_set = resources.lock().unwrap();
    resources_set.clear();
}

/// Finds a value within the request target's list of resources
/// Returns the source of the value as an `Option<String>`
///
/// * `resources` - Address of Mutex HashSet of Strings
/// * `refer` - String of the value that corresponds to the key in search of
///
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

/// Find out if the request target's list of resources is empty
/// Returns a `bool`
///
/// * `resources` - Address of Mutex HashSet of Strings
///
pub fn is_resources_empty(resources: &Arc<Mutex<HashSet<String>>>) -> bool {
    let resources_set = resources.lock().unwrap();
    resources_set.is_empty()
}
