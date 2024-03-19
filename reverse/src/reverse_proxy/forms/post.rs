use hyper::{Body, Client, Method, Response};
use hyper_tls::HttpsConnector;

use std::sync::Arc;

use crate::reverse_proxy::{
    errors::ProxyError, handle_redirection, sessions::process_session, status::bad_request,
};

pub async fn make_post_request(
    action: String,
    params: Vec<(String, String)>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    session: &str,
) -> Result<Response<Body>, ProxyError> {
    let uri = match action.parse() {
        Ok(res) => res,
        Err(_) => action.clone(),
    };

    let post_data = match serde_urlencoded::to_string(params.clone()) {
        Ok(res) => res,
        Err(_) => return bad_request(),
    };

    println!("POST DATA :: {}", post_data);

    let request = hyper::Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", session)
        .body(post_data.into())?;

    let response = client.request(request).await?;
    Ok(response)
}

pub async fn handle_post(
    action: String,
    form_cred: Vec<(String, String)>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    session: &str,
) -> Result<Response<Body>, ()> {
    match make_post_request(action, form_cred, client.clone(), session).await {
        Ok(res) => {
            let new_cookies = process_session(&res.headers().clone());
            if res.status().is_redirection() {
                match handle_redirection(res, client, &new_cookies).await {
                    Ok(new_res) => {
                        return Ok(new_res);
                    }
                    Err(err) => {
                        eprintln!("POST Request error : {}", err);
                        return Err(());
                    }
                }
            }
            Ok(res)
        }
        Err(_) => Err(()),
    }
}
