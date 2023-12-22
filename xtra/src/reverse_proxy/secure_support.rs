use hyper_tls::HttpsConnector;
use native_tls::TlsConnector;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;

pub async fn _https_support(addr: SocketAddr) -> TcpListenerStream {
    let mut cert_buf = Vec::new();
    // let mut key_buf = Vec::new();

    File::open("domain.p12")
        .unwrap()
        .read_to_end(&mut cert_buf)
        .unwrap();

    // File::open("domain.crt").unwrap().read_to_end(&mut cert_buf).unwrap();
    // File::open("domain.key").unwrap().read_to_end(&mut key_buf).unwrap();

    let cert = native_tls::Identity::from_pkcs12(&cert_buf, "httpproxypoc42").unwrap();
    let tls_connector = TlsConnector::builder().identity(cert).build().unwrap();
    let other_https = HttpsConnector::new_with_connector(tls_connector);

    let listener = TcpListener::bind(&addr).await.unwrap();
    let incoming = TcpListenerStream::new(listener);
    incoming
}
