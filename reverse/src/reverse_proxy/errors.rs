use std::error::Error;

#[derive(Debug)]
pub enum ProxyError {
    Hyper(hyper::Error),
    HyperHttp(hyper::http::Error),
}

impl From<hyper::Error> for ProxyError {
    fn from(e: hyper::Error) -> Self {
        ProxyError::Hyper(e)
    }
}

impl From<hyper::http::Error> for ProxyError {
    fn from(e: hyper::http::Error) -> Self {
        ProxyError::HyperHttp(e)
    }
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProxyError::Hyper(_) => write!(fmt, "Hyper: {:?}", self),
            ProxyError::HyperHttp(_) => write!(fmt, "HyperHttp: {:?}", self),
        }
    }
}

impl std::error::Error for ProxyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProxyError::Hyper(e) => Some(e),
            ProxyError::HyperHttp(e) => Some(e),
        }
    }
}
