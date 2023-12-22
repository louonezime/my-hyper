use hyper::{Body, Client, Response};
use hyper_tls::HttpsConnector;

use scraper::{Html, Selector};
use std::sync::Arc;
use toml::Value;

use super::forms::handle_forms;
use super::ProxyError;

// Modified to generalise based on inputs from forms
pub fn _extract_token(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    match Selector::parse("meta[name=csrf-token]") {
        Ok(csrf_token_selector) => {
            let csrf_token = document
                .select(&csrf_token_selector)
                .next()
                .and_then(|meta| meta.value().attr("content"))
                .map(|s| s.to_string());

            csrf_token
        }
        Err(err) => {
            eprintln!("{}", err);
            None
        }
    }
}

async fn process_body(
    body: &[u8],
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    target_url: &str,
    session_cookie: String,
    config: Value,
) -> Result<Response<Body>, ()> {
    let body_str = match String::from_utf8(body.to_owned()) {
        Ok(body) => body,
        Err(err) => {
            eprintln!("{}", err);
            return Err(());
        }
    };

    if body_str.is_empty() || body_str == "" {
        println!("Body not identified/ empty");
    }

    handle_forms(body_str, target_url, client, &session_cookie, config).await
}

pub async fn read_body(
    resp: Response<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    target_url: &str,
    session_cookie: String,
    config: Value,
) -> Result<hyper::Response<Body>, ProxyError> {
    let (parts, body) = resp.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?.to_vec();

    match process_body(&body_bytes, client, target_url, session_cookie, config).await {
        Ok(res) => Ok(res),
        Err(_) => {
            let target_response = Response::from_parts(parts, Body::from(body_bytes));
            Ok(target_response)
        }
    }
}
