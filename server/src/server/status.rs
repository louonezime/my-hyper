use hyper::{Response, Body, StatusCode};

pub fn not_found() -> Result<Response<Body>, hyper::Error> {
    let body = Body::from("Error 404 NOT FOUND: Path not recognised");
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap())
}
