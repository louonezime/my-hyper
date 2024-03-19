use hyper::{
    header::{HeaderMap, HeaderValue, HOST, LOCATION, REFERER},
    server::accept::from_stream,
    service::{make_service_fn, service_fn},
    {Body, Client, Method, Request, Response, Server, Uri},
};
use hyper_tls::HttpsConnector;
use native_tls::TlsConnector;
use tokio_tls::TlsConnector as TokioTlsConnector;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use std::{fs::File, io::Read, net::SocketAddr, str::FromStr, sync::Arc};

use toml::Value;
use url::Url;

mod basic;
mod body;
mod config;
mod cookie;
mod cookie_replacement;
mod errors;
mod forms;
mod secure_support;
mod sessions;
mod status;
mod utils;
use crate::reverse_proxy::{errors::ProxyError, sessions::process_session, status::bad_gateway};

async fn handle_request(
    req: Request<Body>,
    req_uri: &str,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    config: Value,
) -> Result<Response<Body>, ProxyError> {
    let cloned_headers = req.headers().clone();
    let req_method = req.method().clone();
    println!("Request Path: {}", req_uri);

    let req_for_auth =
        match create_new_req(req_uri, req_method, cloned_headers.clone(), Body::empty()).await {
            Some(new_req) => new_req,
            None => return status::unauthorized_response(),
        };
    sessions::detect_cookies(&cloned_headers);

    let response = basic::authenticate(req, client.clone()).await?;
    let resp_headers = response.headers();

    let modified_resp = match basic::intercept_auth(resp_headers, req_for_auth, config).await {
        Ok(res) => client.request(res).await?,
        Err(_) => return Ok(response),
    };
    Ok(modified_resp)
}

pub async fn handle_redirection(
    response: Response<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    cookies: &str,
) -> Result<hyper::Response<Body>, ProxyError> {
    if let Some(new_location) = response.headers().get(LOCATION) {
        if let Ok(new_url) = new_location.to_str() {
            if let Ok(new_uri) = Uri::from_str(new_url) {
                let request = Request::builder()
                    .uri(new_uri)
                    .method("GET")
                    .header("Cookie", cookies)
                    .body(Body::empty())?;

                println!("Redirected");
                return client.request(request).await.map_err(ProxyError::from);
            }
        }
    }

    Ok(response)
}

async fn handle_response(
    req: Request<Body>,
    target_url: &str,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    config: Value,
) -> Result<Response<Body>, ProxyError> {
    let mut target_response =
        handle_request(req, target_url, client.clone(), config.clone()).await?;

    if target_response.status().is_redirection() {
        target_response = handle_redirection(target_response, client.clone(), "").await?;
    }
    let resp_header = target_response.headers();
    let session_cookie = process_session(resp_header);
    target_response =
        body::read_body(target_response, client, target_url, session_cookie, config).await?;

    sessions::handle_cookies(target_response.headers());

    println!("RESP STATUS\t{:?}", target_response.status());
    println!("RESP EXTENSIONS\t{:?}", target_response.extensions());
    println!("\nTarget Server Response Headers:");
    utils::print_formatted_headers(target_response.headers());

    Ok(target_response)
}

async fn create_new_req(
    target_url: &str,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Option<Request<Body>> {
    let url = Url::parse(target_url)
        .map_err(|err| {
            eprintln!("Failed to parse URL: {}", err);
        })
        .ok()?;
    let mut target_request = Request::new(body);

    *target_request.uri_mut() = hyper::Uri::from_str(target_url)
        .map_err(|err| {
            eprintln!("Faied to parse URI: {}", err);
        })
        .ok()?;
    *target_request.method_mut() = method;
    *target_request.headers_mut() = headers;

    if let Some(host) = url.host_str() {
        if let Ok(new_host) = HeaderValue::from_str(host) {
            if target_request
                .headers_mut()
                .insert(HOST, new_host)
                .is_some()
            {
                return Some(target_request);
            }
        }
    }
    None
}

async fn reverse_proxy(
    req: Request<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    path: String,
    config: Value,
) -> Result<Response<Body>, ProxyError> {
    let headers = req.headers().clone();
    let method = req.method().clone();
    let (parts, body) = req.into_parts();

    let target_url = match utils::determine_target(&path, &parts.uri, config.clone()) {
        Ok(url) => url,
        Err(_) => return status::not_found(),
    };

    let target_request = match create_new_req(&target_url, method, headers, body).await {
        Some(new_req) => new_req,
        None => return status::bad_request(),
    };

    let target_response = handle_response(target_request, &target_url, client, config).await?;
    Ok(target_response)
}

async fn handle(
    req: Request<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    mut path: String,
) -> Result<Response<Body>, ProxyError> {
    let config = match config::define_conf("./config.toml") {
        Ok(parsed_toml) => parsed_toml,
        Err(_) => return bad_gateway("config.toml"),
    };
    let req_headers = req.headers();

    if let Some(ref_header) = req_headers.get(REFERER) {
        path = match ref_header.to_str() {
            Ok(res) => res.to_string(),
            Err(_) => path,
        }
    }
    println!("REQ VERSION\t {:?}", req.version());
    println!("REQ EXTENSIONS\t {:?}", req.extensions());
    println!("\nProxy Request Headers:");
    utils::print_formatted_headers(req_headers);

    let target_response = reverse_proxy(req, client, path, config).await?;
    Ok(target_response)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

// fn configure_tls(cert: &[u8], key: &[u8]) -> ServerConfig {
//     let mut config = ServerConfig::new(tokio_rustls::rustls::NoClientAuth::new());
//     let cert_chain = tokio_rustls::rustls::internal::pemfile::certs(&mut cert.as_slice()).unwrap();
//     let key_chain = tokio_rustls::rustls::internal::pemfile::rsa_private_keys(&mut key.as_slice()).unwrap();
//     config.set_single_cert(cert_chain, key_chain[0].clone()).expect("Failed to set certificate and private key");
//     config
// }

#[tokio::main]
pub async fn create_serv() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3128));

    // ---- HTTPS SUPPORT ----
    let mut cert_buf = Vec::new();
    File::open("domain.p12")
        .unwrap()
        .read_to_end(&mut cert_buf)
        .unwrap();

    let tls_connector = TlsConnector::builder().build().unwrap();
    let tokio_tls_connector = TokioTlsConnector::from(tls_connector);
    let hyper_tls_connector = hyper_tls::TlsConnector::from(tokio_tls_connector);
    let https_connector = HttpsConnector::new_with_connector(hyper_tls_connector);

    let listener = TcpListener::bind(&addr).await.unwrap();
    let incoming = TcpListenerStream::new(listener);

    let client = Arc::new(hyper::Client::builder().build::<_, hyper::Body>(https_connector));
    let client_for_service = client.clone();
    // -----------------------

    // let https = HttpsConnector::new();
    // let client = Arc::new(Client::builder().build(https));
    // let client_for_service = client.clone();

    let make_proxy_svc = make_service_fn(move |_conn| {
        let client = client_for_service.clone();
        async {
            Ok::<_, ProxyError>(service_fn(move |req| {
                let client = client.clone();
                let path = req.uri().path().to_owned();

                println!("Path: {}", path);
                handle(req, client, path)
            }))
        }
    });

    // ---- HTTPS SUPPORT ----

    println!("Reverse proxy listening on https://{}", addr);
    let proxy_server = Server::builder(from_stream(incoming))
        .serve(make_proxy_svc)
        .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = proxy_server.await {
        eprintln!("Server Error:\n\t{}", e);
    } else {
        println!("\nServer shutdown gracefully!")
    }
    // -----------------------

    // println!("Reverse proxy listening on http://{}", addr);
    // let proxy_server = Server::bind(&addr).serve(make_proxy_svc);
    // let graceful = proxy_server.with_graceful_shutdown(shutdown_signal());
    // if let Err(e) = graceful.await {
    //     eprintln!("Server Error:\n\t{}", e);
    // } else {
    //     println!("\nServer shutdown gracefully!")
    // }
}
