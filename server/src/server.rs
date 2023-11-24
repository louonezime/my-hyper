use hyper::header::REFERER;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};

use hyper_tls::HttpsConnector;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

mod map;
mod status;

async fn _shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn handle_response(
    req: Request<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
) -> Result<Response<Body>, hyper::Error> {
    let target_response = client.request(req).await?;

    println!(
        "Target Server Response Status: {}",
        target_response.status()
    );
    println!(
        "Target Server Response Headers: {:?}\n",
        target_response.headers()
    );

    Ok(target_response)
}

async fn reverse_proxy(
    req: Request<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    path: String,
) -> Result<Response<Body>, hyper::Error> {
    let headers = req.headers().clone();
    let method = req.method().clone();
    let (parts, body) = req.into_parts();

    let target_url = match map::determine_target(path, &parts.uri) {
        Ok(url) => url,
        Err(_) => return status::not_found(),
    };

    let mut target_request = Request::new(body);
    *target_request.uri_mut() = hyper::Uri::from_str(target_url.as_str()).unwrap();
    *target_request.method_mut() = method;
    *target_request.headers_mut() = headers;

    let target_response = handle_response(target_request, client).await?;
    Ok(target_response)
}

async fn handle(
    req: Request<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    mut path: String,
) -> Result<Response<Body>, hyper::Error> {
    let req_headers = req.headers().clone();

    if let Some(ref_header) = req_headers.get(REFERER) {
        path = match ref_header.to_str() {
            Ok(res) => res.to_string(),
            Err(_) => path,
        }
    }
    println!("Proxy Request Headers: {:?}\n", req_headers);

    let target_response = reverse_proxy(req, client, path).await?;
    Ok(target_response)
}

#[tokio::main]
pub async fn create_serv() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3128));

    let https = HttpsConnector::new();
    let client = Arc::new(Client::builder().build(https));
    let client_for_service = client.clone();

    let make_svc = make_service_fn(move |_conn| {
        let client = client_for_service.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let client = client.clone();
                let path = req.uri().path().to_string();

                println!("Path: {}", path);
                handle(req, client, path)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);

    println!("Reverse proxy listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("Server Error: {}", e);
    }
}
