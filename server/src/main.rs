use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

async fn hello(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello there")))
}

#[tokio::main]
async fn main() {
    let sock_addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_servc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello))
    });

    let server = Server::bind(&sock_addr).serve(make_servc);

    if let Err(e) = server.await {
        eprintln!("Server Error: {}", e);
    }
}
