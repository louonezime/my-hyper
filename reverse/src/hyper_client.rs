use hyper::{client::connect::HttpConnector, Client};
use hyper_tls::HttpsConnector;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
pub async fn packet_response(url: &str) -> Result<()> {
    let url = url.parse::<hyper::Uri>()?;
    if url.scheme_str() != Some("http") && url.scheme_str() != Some("https") {
        println!("Error: Invalid scheme");
        return Ok(());
    }

    fetch_url(url).await
}

async fn fetch_url(url: hyper::Uri) -> Result<()> {
    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>> = Client::builder().build(https);

    let res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}", res.headers());
    println!("Version: {:?}", res.version());
    println!("Extensions: {:?}", res.extensions());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_fetch_url_exec() {
        let url = "https://www.google.com".parse().unwrap();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let result = fetch_url(url).await;
            assert!(result.is_ok());
        });
    }
}
