use hyper::header::{Entry::Occupied, HeaderMap, HeaderName, HeaderValue, COOKIE, SET_COOKIE};

use std::str;

pub fn process_session(headers: &HeaderMap) -> String {
    let mut final_cookie = String::new();
    let cookie_headers = headers.get_all(SET_COOKIE);

    for headers in cookie_headers.iter() {
        if let Ok(cookie_value) = headers.to_str() {
            let cookies: Vec<&str> = cookie_value.split(';').collect();

            final_cookie.push_str(cookies[0]);
            final_cookie.push_str("; ");
        }
    }
    final_cookie
}

fn process_cookies(cookie_value: &str) {
    let cookies: Vec<&str> = cookie_value.split(';').collect();

    for cookie in cookies {
        println!(" Individual Cookie: {}", cookie);
    }
}

// Header from Response<Body> for the cookies set by the response
pub fn handle_cookies(headers: &HeaderMap) {
    if let Some(cookie_header) = headers.get(SET_COOKIE) {
        println!("Cookies set!");
        if let Ok(cookie_value) = cookie_header.to_str() {
            process_cookies(cookie_value);
        }
        println!();
    } else {
        println!("No cookies are set in the response\n");
    }
}

// Header from Request<Body> for existing cookies in request
pub fn detect_cookies(headers: &HeaderMap) {
    if let Some(cookie_header) = headers.get(COOKIE) {
        println!("Cookies found!");
        if let Ok(cookie_value) = cookie_header.to_str() {
            process_cookies(cookie_value);
        }
        println!();
    } else {
        println!("No cookies found in the request headers.");
    }
}

struct CookieReplacement {
    key: Vec<u8>,
    value: Vec<u8>,
    replacement: Vec<u8>,
}

fn cookie_replacement(value: &mut HeaderValue, replacements: &[CookieReplacement]) -> Vec<u8> {
    println!("HeaderValue: {}", value.to_str().unwrap());
    for cookie_replacement in replacements {
        println!(
            "  Replacement ({}): {} -> {}",
            str::from_utf8(&cookie_replacement.key).unwrap(),
            str::from_utf8(&cookie_replacement.value).unwrap(),
            str::from_utf8(&cookie_replacement.replacement).unwrap()
        );
    }

    Vec::new()
}

fn header_map_update(
    map: &mut HeaderMap,
    header: HeaderName,
    replacements: &[CookieReplacement],
) -> () {
    if let Occupied(mut entry) = map.entry(header) {
        for entry in entry.iter_mut() {
            cookie_replacement(entry, replacements);
        }
    }
}

/*
TODO Limitations: HeaderMap can store a maximum of 32,768 headers (header name / value pairs). Attempting to insert more will result in a panic.
https://docs.rs/hyper/0.14.27/hyper/header/index.html
*/

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::{COOKIE, SET_COOKIE};

    #[test]
    fn test_cookie_replacement() {
        let mut map = HeaderMap::default();
        map.insert(SET_COOKIE, HeaderValue::from_static("world"));
        map.append(SET_COOKIE, HeaderValue::from_static("earth"));
        map.append(COOKIE, HeaderValue::from_static("earth"));

        header_map_update(
            &mut map,
            SET_COOKIE,
            &[CookieReplacement {
                key: vec![0x42],
                value: vec![0x44],
                replacement: vec![0x45],
            }],
        )
    }
}
