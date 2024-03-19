use hyper::{
    header::{CONTENT_TYPE, WWW_AUTHENTICATE},
    Body, Response, StatusCode,
};

use super::ProxyError;

pub fn not_found() -> Result<Response<Body>, ProxyError> {
    let body = Body::from("Error 404 NOT FOUND: Path not recognised");

    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .body(body)?)
}

pub fn unauthorized_response() -> Result<Response<Body>, ProxyError> {
    let body = Body::from("Error 401 UNAUTHORIZED: Authentication required\n");

    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(WWW_AUTHENTICATE, "Basic realm=\"Authentication required\"")
        .body(body)?)
}

pub fn bad_request() -> Result<Response<Body>, ProxyError> {
    let body = Body::from("Error 400 BAD REQUEST: Proxy config file not properly completed");

    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(CONTENT_TYPE, "text/plain")
        .body(body)?)
}

pub fn bad_gateway(specification: &str) -> Result<Response<Body>, ProxyError> {
    let mut body = Body::from("Error 502 BAD GATEWAY");

    if specification == "config.toml" {
        body = Body::from("Error 502 BAD GATEWAY: Proxy config isn't recognized");
    }

    Ok(Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .header(CONTENT_TYPE, "text/plain")
        .body(body)?)
}
