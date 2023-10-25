use hyper::{Client, Response, Body};
use hyper::client::connect::HttpConnector;
use hyper_tls::HttpsConnector;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
pub async fn packet_response(url: &str) -> Result<()> {
    let url = url.parse::<hyper::Uri>().unwrap();
    if url.scheme_str() == Some("https") {
        fetch_secured_url(url).await
    } else if url.scheme_str() == Some("http") {
        fetch_url(url).await
    } else {
        eprintln!("Error: Pass a valid HTTP URL as an argument");
        Ok(())
    }
}

fn print_result(res: Response<Body>) {
    println!("Response: {}", res.status());
    println!("Headers: {:#?}", res.headers());
    println!("Version: {:?}", res.version());
    // May be useful, but always prints Extensions
    println!("Extensions: {:?}", res.extensions());
}

async fn fetch_secured_url(url: hyper::Uri) -> Result<()> {
    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>> = Client::builder().build(https);

    let res = client.get(url).await?;
    print_result(res);

    Ok(())
}

async fn fetch_url(url: hyper::Uri) -> Result<()> {
    let client = Client::new();

    let res = client.get(url).await?;
    print_result(res);

    Ok(())
}
