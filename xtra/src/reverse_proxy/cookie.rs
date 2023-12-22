use chrono::{DateTime, Utc};
use std::str;

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie
// https://httpwg.org/specs/rfc6265.html

#[derive(Debug, PartialEq, Hash)]
pub struct SetCookie<'a> {
    name: &'a [u8],
    value: &'a [u8],
    attrs: SetCookieAttributes<'a>,
}

#[derive(Debug, PartialEq, Hash)]
pub struct SetCookieAttributes<'a> {
    max_age: Option<u32>,
    domain: &'a [u8],
    path: &'a [u8],
    same_site: SameSite,
    secure: bool,
    http_only: bool,
    partitioned: bool,
}

#[derive(Debug, PartialEq, Hash)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

pub struct CookieIterator<'a> {
    slice: &'a [u8],
}

/// Builds an iterator on the HTTP Cookie header.
///
/// # Exemple
///
/// ```
/// for (name, value) in read_cookies(b"key1=value1; key2=value2") {
///     let name = std::str::from_utf8(name).unwrap();
///     let value = std::str::from_utf8(value).unwrap();
///     println!("name={} value={}", name, value);
/// }
/// ```
pub fn read_cookies<'a>(data: &'a [u8]) -> CookieIterator<'a> {
    CookieIterator::new(data)
}

/// Parse HTTP header Set-Cookie.
///
/// # Exemple
///
/// ```
/// if let Some(cookie) = parse_set_cookie(b"name=value; Max-Age=3600", timestamp()) {
///     let name = std::str::from_utf8(cookie.name).unwrap();
///     let value = std::str::from_utf8(cookie.value).unwrap();
///     let max_age = cookie.attrs.max_age.unwrap_or(0);
///     println!("name={} value={} max-age={}", name, value, max_age);
/// }
/// ```
pub fn parse_set_cookie<'a>(data: &'a [u8], timestamp: u64) -> Option<SetCookie> {
    if let Some((name, rest)) = SetCookie::parse_name(data) {
        if let Some((value, attrs)) = SetCookie::parse_value(rest) {
            let attrs = parse_set_cookie_attribute(attrs, timestamp);
            return Some(SetCookie { name, value, attrs });
        }
        return Some(SetCookie {
            name,
            value: to_end_slice(data),
            attrs: Default::default(),
        });
    }
    None
}

impl<'a> SetCookie<'a> {
    fn parse_name(data: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        let data = ltrim(data);
        if let Some((name, rest)) =
            split_once_fn(data, |c| *c == b';' || *c == b'=', |c| *c == b'=')
        {
            let name = rtrim(name);
            if !name.is_empty() {
                return Some((name, rest));
            }
        }
        return None;
    }

    fn parse_value(data: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        let data = ltrim(data);
        if let Some((value, attrs)) = split_once(data, b';') {
            let value = rtrim(value);
            return Some((value, attrs));
        }
        let value = rtrim(data);
        return Some((value, to_end_slice(data)));
    }
}

impl<'a> CookieIterator<'a> {
    fn new(slice: &'a [u8]) -> Self {
        Self { slice }
    }

    pub fn remaining(&self) -> &'a [u8] {
        self.slice
    }
}

impl<'a> Iterator for CookieIterator<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((name, mut value)) = split_once(self.slice, b'=') {
            if let Some((value2, rest)) = split_once(value, b';') {
                value = value2;
                self.slice = rest;
            } else {
                self.slice = to_end_slice(self.slice);
            }
            return Some((trim(name), trim(value)));
        }
        self.slice = to_end_slice(self.slice);
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if !self.slice.is_empty() {
            return (0, Some(self.slice.len()));
        }
        (0, Some(0))
    }
}

impl<'a> Default for SetCookieAttributes<'a> {
    fn default() -> SetCookieAttributes<'a> {
        Self {
            max_age: None,
            domain: &[],
            path: &[],
            same_site: SameSite::Lax,
            secure: false,
            http_only: false,
            partitioned: false,
        }
    }
}

fn parse_set_cookie_attribute<'a>(data: &'a [u8], timestamp: u64) -> SetCookieAttributes {
    let mut attrs: SetCookieAttributes = Default::default();
    let mut buffer = [0u8; 12];
    let mut has_max_age = false;

    for part in data.split(|c| c == &b';') {
        let part = trim(part);

        let (name, value) = if let Some((name, value)) = split_once(part, b'=') {
            // remove quote
            match ltrim(value) {
                [b'"', rest @ .., b'"'] => (rtrim(name), ltrim(rest)),
                [b'"', ..] => break,
                [rest @ ..] => (rtrim(name), ltrim(rest)),
            }
        } else {
            (part, to_end_slice(part))
        };

        match to_limited_upper(name, &mut buffer) {
            b"MAX-AGE" => {
                if let Some(max_age) = parse_max_age(value) {
                    has_max_age = true;
                    attrs.max_age = Some(max_age);
                }
            }
            b"EXPIRES" if !has_max_age => {
                if let Some(max_age) = parse_expires(value, timestamp) {
                    attrs.max_age = Some(max_age);
                }
            }
            b"DOMAIN" => match value {
                [] => {}
                [b'.', rest @ ..] => attrs.domain = rest,
                _ => attrs.domain = value,
            },
            b"PATH" => match value {
                [b'/', ..] => attrs.path = value,
                _ => {}
            },
            b"SECURE" => attrs.secure = true,
            b"HTTPONLY" => attrs.http_only = true,
            b"PARTITIONED" => attrs.partitioned = true,
            b"SAMESITE" => {
                attrs.same_site = match to_limited_upper(value, &mut buffer) {
                    b"NONE" => SameSite::None,
                    b"STRICT" => SameSite::Strict,
                    _ /* | b"LAX"*/ => SameSite::Lax,
                }
            }
            _ => {}
        }
    }

    attrs
}

fn split_once(data: &[u8], sep: u8) -> Option<(&[u8], &[u8])> {
    split_once_fn(data, |c| *c == sep, |_| true)
}

fn split_once_fn<P, C>(data: &[u8], pred: P, check: C) -> Option<(&[u8], &[u8])>
where
    P: FnMut(&u8) -> bool,
    C: FnOnce(&u8) -> bool,
{
    if let Some(position) = data.iter().position(pred) {
        let sep = unsafe { data.get_unchecked(position) };
        if check(sep) {
            return Some((&data[..position], &data[position + 1..]));
        }
    }
    None
}

fn to_end_slice(data: &[u8]) -> &[u8] {
    let n = data.len();
    &data[n..n]
}

trait IndexIf<T> {
    fn index_if<P>(&self, pred: P) -> usize
    where
        P: FnMut(T) -> bool;
}

trait RIndexIf<T> {
    fn rindex_if<P>(&self, pred: P) -> usize
    where
        P: FnMut(T) -> bool;
}

impl<'a, T: 'a> IndexIf<&'a T> for &'a [T] {
    fn index_if<P>(&self, mut pred: P) -> usize
    where
        P: FnMut(&'a T) -> bool,
    {
        let mut position = 0;
        while position < self.len() && pred(unsafe { self.get_unchecked(position) }) {
            position += 1;
        }
        position
    }
}

impl<'a, T: 'a> RIndexIf<&'a T> for &'a [T] {
    fn rindex_if<P>(&self, mut pred: P) -> usize
    where
        P: FnMut(&'a T) -> bool,
    {
        let mut position = self.len();
        while position > 0 && pred(unsafe { self.get_unchecked(position - 1) }) {
            position -= 1;
        }
        position
    }
}

fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

fn ltrim(data: &[u8]) -> &[u8] {
    &data[data.index_if(|c| is_whitespace(*c))..]
}

fn rtrim(data: &[u8]) -> &[u8] {
    &data[..data.rindex_if(|c| is_whitespace(*c))]
}

fn trim(data: &[u8]) -> &[u8] {
    rtrim(ltrim(data))
}

fn parse_max_age(max_age: &[u8]) -> Option<u32> {
    let (max_age, is_neg) = match max_age {
        [] | [b'-'] => return None,
        [b'-', rest @ ..] => (rest, true),
        [rest @ ..] => (rest, false),
    };

    let mut delta = 0u32;

    for c in max_age {
        // invalid number
        if c < &b'0' || &b'9' < c {
            return None;
        }

        // no overflow for very large value, but not break for checked all digits
        if delta <= u32::MAX / 11 {
            delta *= 10;
            delta += (c - b'0') as u32;
        }
    }

    if is_neg {
        delta = 0;
    }
    Some(delta)
}

fn parse_expires(expires: &[u8], timestamp: u64) -> Option<u32> {
    if let Ok(date) = str::from_utf8(&expires) {
        // Wed, 21 Oct 2015 07:28:00 GMT
        if let Ok(gmt_time) = DateTime::parse_from_rfc2822(date) {
            let utc_time = gmt_time.with_timezone(&Utc);
            let end_time = utc_time.timestamp();
            if end_time <= 0 || end_time as u64 <= timestamp {
                return Some(0);
            }

            let duration = end_time as u64 - timestamp;
            if duration > u32::MAX as u64 {
                return Some(u32::MAX);
            }
            return Some(duration as u32);
        }
    }
    None
}

const ALPHA_UPPER_TABLE: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'-' as usize] = b'-';

    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = c - b'a' + b'A';
        c += 1;
    }

    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = c;
        c += 1;
    }

    t
};

fn to_limited_upper<'a>(s: &[u8], buffer: &'a mut [u8]) -> &'a [u8] {
    if s.len() > buffer.len() {
        return &[];
    }

    let len = s.len();
    for i in 0..len {
        let src = unsafe { s.get_unchecked(i) };
        let dst = unsafe { buffer.get_unchecked_mut(i) };
        let upper = unsafe { ALPHA_UPPER_TABLE.get_unchecked(*src as usize) };
        *dst = *upper;
    }

    &buffer[0..len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() {
        assert_eq!(ltrim(b"abc"), b"abc");
        assert_eq!(ltrim(b"  abc"), b"abc");
        assert_eq!(ltrim(b"abc  "), b"abc  ");
        assert_eq!(ltrim(b"  abc  "), b"abc  ");
        assert_eq!(ltrim(b"  "), b"");
        assert_eq!(ltrim(b""), b"");

        assert_eq!(rtrim(b"abc"), b"abc");
        assert_eq!(rtrim(b"  abc"), b"  abc");
        assert_eq!(rtrim(b"abc  "), b"abc");
        assert_eq!(rtrim(b"  abc  "), b"  abc");
        assert_eq!(rtrim(b"  "), b"");
        assert_eq!(rtrim(b""), b"");

        assert_eq!(trim(b"abc"), b"abc");
        assert_eq!(trim(b"  abc"), b"abc");
        assert_eq!(trim(b"abc  "), b"abc");
        assert_eq!(trim(b"  abc  "), b"abc");
        assert_eq!(trim(b"  "), b"");
        assert_eq!(trim(b""), b"");
    }

    #[test]
    fn test_parse_max_age() {
        assert_eq!(parse_max_age(b"0"), Some(0));
        assert_eq!(parse_max_age(b"123"), Some(123));
        assert_eq!(parse_max_age(b"-123"), Some(0));
        assert_eq!(
            parse_max_age(b"99999999999999999999999999999"),
            Some(999999999)
        );
        assert_eq!(parse_max_age(b"123abc"), None);
    }

    #[test]
    fn test_parse_set_cookie_attribute() {
        assert_eq!(
            parse_set_cookie_attribute(b"MAX-AGE=123; PATH=/foo/bar; DOMAIN=domain.org; EXPIRES=Wed, 21 Oct 2015 07:28:00 GMT; HTTPONLY; SECURE; SAMESITE=Strict; PARTITIONED", 0),
            SetCookieAttributes{
                max_age: Some(123),
                domain: b"domain.org",
                path: b"/foo/bar",
                same_site: SameSite::Strict,
                secure: true,
                http_only: true,
                partitioned: true,
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"max-age=123; path=\"/foo/bar\"; domain=domain.org; expires=Wed, 21 Oct 2015 07:28:00 GMT; httponly; secure; samesite=strict; partitioned", 0),
            SetCookieAttributes{
                max_age: Some(123),
                domain: b"domain.org",
                path: b"/foo/bar",
                same_site: SameSite::Strict,
                secure: true,
                http_only: true,
                partitioned: true,
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Max-Age=abc; Path=/foo/bar; Path=bar; Domain=domain.org; HttpOnly; Secure; SameSite=Strict; Partitioned", 0),
            SetCookieAttributes{
                max_age: None,
                domain: b"domain.org",
                path: b"/foo/bar",
                same_site: SameSite::Strict,
                secure: true,
                http_only: true,
                partitioned: true,
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Max-Age=123; Max-Age=abc; Path=/foo/bar; Path=/bar", 0),
            SetCookieAttributes {
                max_age: Some(123),
                domain: b"",
                path: b"/bar",
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Max-Age=123; Path=foo/bar; Max-Age = 432 ; ", 0),
            SetCookieAttributes {
                max_age: Some(432),
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Expires=Wed, 21 Oct 2015 07:28:00 GMT; Max-Age=123; ", 0),
            SetCookieAttributes {
                max_age: Some(123),
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Expires=Wed, 21 Oct 2015 07:28:00 GMT;", 0),
            SetCookieAttributes {
                max_age: Some(1445412480),
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Expires=Wed, 21 Oct 2015 07:28:00 GMT;", 2_000_000_000),
            SetCookieAttributes {
                max_age: Some(0),
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Path=", 0),
            SetCookieAttributes {
                max_age: None,
                domain: b"",
                path: b"",
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Max-Age=123; Path=\"/foo/bar Domain=org\"", 0),
            SetCookieAttributes {
                max_age: Some(123),
                domain: b"",
                path: b"/foo/bar Domain=org",
                ..Default::default()
            }
        );

        assert_eq!(
            parse_set_cookie_attribute(b"Max-Age=123; Path=\"/foo/bar Domain=org", 0),
            SetCookieAttributes {
                max_age: Some(123),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_parse_set_cookie() {
        assert_eq!(
            parse_set_cookie(b"Path=", 0),
            Some(SetCookie {
                name: b"Path",
                value: b"",
                attrs: Default::default()
            })
        );

        assert_eq!(
            parse_set_cookie(b" Path = abc ", 0),
            Some(SetCookie {
                name: b"Path",
                value: b"abc",
                attrs: Default::default()
            })
        );

        assert_eq!(
            parse_set_cookie(b"Path=/value; Path=/a", 0),
            Some(SetCookie {
                name: b"Path",
                value: b"/value",
                attrs: SetCookieAttributes {
                    max_age: None,
                    domain: b"",
                    path: b"/a",
                    ..Default::default()
                },
            })
        );

        assert_eq!(
            parse_set_cookie(b" Path = abc ; Path = /a", 0),
            Some(SetCookie {
                name: b"Path",
                value: b"abc",
                attrs: SetCookieAttributes {
                    max_age: None,
                    domain: b"",
                    path: b"/a",
                    ..Default::default()
                },
            })
        );

        assert_eq!(parse_set_cookie(b"", 0), None);
        assert_eq!(parse_set_cookie(b"=; Path=/a", 0), None);
        assert_eq!(parse_set_cookie(b" = ; Path=/a", 0), None);
        assert_eq!(parse_set_cookie(b"; Path=/a", 0), None);
    }

    #[test]
    fn test_cookie_iter() {
        type S = &'static [u8];

        assert_eq!(
            read_cookies(b"key=value; key2=value2; key=value").collect::<Vec<_>>(),
            [
                (b"key" as S, b"value" as S),
                (b"key2", b"value2"),
                (b"key", b"value")
            ]
        );

        assert_eq!(
            read_cookies(b" key = value ; key2 = value2 ; key = value ").collect::<Vec<_>>(),
            [
                (b"key" as S, b"value" as S),
                (b"key2", b"value2"),
                (b"key", b"value")
            ]
        );

        assert_eq!(
            read_cookies(b" key =  ").collect::<Vec<_>>(),
            [(b"key" as S, b"" as S)]
        );
        assert_eq!(
            read_cookies(b"  =  ").collect::<Vec<_>>(),
            [(b"" as S, b"" as S)]
        );
        assert_eq!(
            read_cookies(b"=").collect::<Vec<_>>(),
            [(b"" as S, b"" as S)]
        );
        assert_eq!(read_cookies(b"key").collect::<Vec<_>>(), []);
        assert_eq!(read_cookies(b"").collect::<Vec<_>>(), []);
    }
}
