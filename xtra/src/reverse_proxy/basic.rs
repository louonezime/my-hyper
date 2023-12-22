use http::{HeaderMap, HeaderValue};
use hyper::{
    header::{AUTHORIZATION, WWW_AUTHENTICATE},
    Body, Client, Request, Response,
};
use hyper_tls::HttpsConnector;

use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use toml::Value;

use super::{config::setup_basic, ProxyError};
use crate::reverse_proxy::utils::{clean_url, common_prefix};

#[derive(Debug)]
pub struct ServerCredentials {
    pub realm: String,
    pub username: String,
    pub password: String,
}

impl ServerCredentials {
    pub fn new(realm: &str, username: &str, password: &str) -> Self {
        ServerCredentials {
            realm: realm.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    fn _print_credentials(&self) {
        println!(
            "|**\n|Realm: {}\n|Username: {}\n|Password: {}\n|_",
            self.realm, self.username, self.password
        );
    }
}

fn _decode_auth(auth_val: &str) -> Result<(String, String), &'static str> {
    let decoded_auth = general_purpose::STANDARD
        .decode(auth_val)
        .map_err(|_| "Base64 decoding error")?;
    let auth_str = String::from_utf8(decoded_auth).map_err(|_| "UTF-8 decoding error")?;
    let auth_parts: Vec<&str> = auth_str.split(':').collect();

    if auth_parts.len() == 2 {
        Ok((auth_parts[0].to_string(), auth_parts[1].to_string()))
    } else {
        Err("Invalid format")
    }
}

fn encode_auth(user: String, password: String) -> String {
    let combined_auth = format!("{}:{}", user, password);
    let encoded_credentials = general_purpose::STANDARD.encode(combined_auth);

    let encoded_auth = format!("Basic {}", encoded_credentials);
    encoded_auth
}

async fn reconstruct_req(
    req: Request<Body>,
    info: &ServerCredentials,
) -> Result<Request<Body>, ProxyError> {
    let (parts, body) = req.into_parts();
    let mut authenticated_req = Request::from_parts(parts, body);
    let auth = encode_auth(info.username.clone(), info.password.clone());

    if let Ok(auth_value) = HeaderValue::from_str(&auth) {
        authenticated_req
            .headers_mut()
            .insert(AUTHORIZATION, auth_value);
    }
    println!("Authentication request sent!");
    println!(" |__ {}\n", format_args!("{}: {}", "authorization", auth));

    Ok(authenticated_req)
}

fn match_credentials<'a>(
    uri: &str,
    auth_map: &'a HashMap<String, ServerCredentials>,
) -> Option<&'a ServerCredentials> {
    match auth_map.get(uri) {
        Some(_) => return auth_map.get(uri),
        None => {
            for (key, _) in auth_map.iter() {
                if common_prefix(uri, key) == *key {
                    return auth_map.get(key);
                }
            }
        }
    }

    None
}

async fn auth_middleware(
    req: Request<Body>,
    realm: &str,
    config: Value,
) -> Result<Request<Body>, ProxyError> {
    let basic_auth = match setup_basic(config) {
        Some(map) => map,
        None => return Ok(req),
    };
    let target = clean_url(&(req.uri().to_string()));

    let credential_info = match match_credentials(&target, &basic_auth) {
        Some(res) => res,
        None => return Ok(req),
    };

    if realm == credential_info.realm {
        let authenticated = reconstruct_req(req, credential_info).await?;
        return Ok(authenticated);
    }
    Ok(req)
}

pub async fn intercept_auth(
    resp_headers: &HeaderMap,
    cloned_req: Request<Body>,
    config: Value,
) -> Result<Request<Body>, ProxyError> {
    let auth_header = resp_headers.get(WWW_AUTHENTICATE);
    let auth_str = auth_header
        .and_then(|header| header.to_str().ok())
        .unwrap_or_default();

    if let Some(basic) = auth_str.strip_prefix("Basic realm=\"") {
        if let Some(end_quote_index) = basic.find('\"') {
            let extracted_auth = &basic[..end_quote_index];
            println!("Identified Basic Authentication needed!");
            println!(" |__ {}\n", extracted_auth);

            let authenticated_req = auth_middleware(cloned_req, extracted_auth, config).await?;
            return Ok(authenticated_req);
        } else {
            eprintln!("Realm Parsing Error: Closing double quote not found!");
        }
    }
    Ok(cloned_req)
}

pub async fn authenticate(
    req: Request<Body>,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
) -> Result<Response<Body>, ProxyError> {
    let headers = req.headers();

    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        println!("Authorization Header found!");
        if let Ok(basic) = auth_header.to_str() {
            if let Some(basic_str) = basic.strip_prefix("Basic ") {
                println!(" Basic Authorization Header: {:?}\n", basic_str);
            }
        }
    } else {
        println!("No Authorization Header identified\n");
    }
    let response = client.request(req).await?;
    Ok(response)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_encode_auth_valid() {
        let test_cases = vec![
            (("foo", "bar"), "Basic Zm9vOmJhcg=="),
            (("user", "password"), "Basic dXNlcjpwYXNzd29yZA=="),
        ];

        for ((user, pwd), expected) in test_cases {
            let result = encode_auth(user.to_string(), pwd.to_string());
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_decode_valid_cases() {
        let test_cases = vec![
            ("Zm9vOmJhcg==", ("foo", "bar")),
            ("dXNlcjpwYXNzd29yZA==", ("user", "password")),
        ];

        for (decoded, (user, pwd)) in test_cases {
            let (res_user, res_pwd) = match _decode_auth(decoded) {
                Ok(res) => res,
                Err(_) => ("user".to_string(), "password".to_string()),
            };
            assert_eq!((res_user, res_pwd), (user.to_string(), pwd.to_string()));
        }
    }

    #[test]
    fn test_decode_invalid_base64() {
        assert_eq!(
            _decode_auth("dXNlcm5hbWU6cGFzc3dvcmQ").unwrap_err(),
            "Base64 decoding error"
        );
    }

    #[test]
    fn test_decode_invalid_utf8() {
        let result = _decode_auth("dXNlcm5hbWU6cGFzc3dvcmQ==");
        assert!(result.is_err());
        match result {
            Ok((user, pass)) => {
                assert!(user.is_ascii());
                assert!(pass.is_ascii());
            }
            Err(err) => assert_eq!("Base64 decoding error", err),
        }
    }

    #[test]
    fn test_decode_invalid_format() {
        assert_eq!(
            _decode_auth("dXNlcm5hbWU6cGFzc3dvcmQ6cGFzc3dvcmQ=").unwrap_err(),
            "Invalid format"
        );
    }
}
