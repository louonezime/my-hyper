use hyper::{Response, Body};

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

pub mod resources;
pub mod external_links;

fn filter_resources_links(
    links: &HashSet<String>, sources: &Arc<Mutex<HashSet<String>>>) {
    let mut resources = sources.lock().unwrap();

    resources.retain(|resource| !links.contains(resource));
}

fn process_body(
    body: &Vec<u8>, resources: &Arc<Mutex<HashSet<String>>>, proxy_path: &str) {
    match String::from_utf8(body.clone()) {
        Ok(body_str) => {
            resources::check_resources(&body_str, resources);
        },
        Err(_) => eprintln!("Warning: Non-UTF-8 data encountered.\n")
    }
}

pub async fn request_routing(
    resp: hyper::Response<Body>,
    path: &str,
    resources: &Arc<Mutex<HashSet<String>>>
) -> Result<hyper::Response<Body>, hyper::Error> {
    let (parts, body) = resp.into_parts();

    let body_bytes = hyper::body::to_bytes(body)
        .await?
        .to_vec();

    process_body(&body_bytes, resources, path);

    let target_response = Response::from_parts(parts, Body::from(body_bytes));
    Ok(target_response)
}
