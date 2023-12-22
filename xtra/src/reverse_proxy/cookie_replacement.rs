use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use std::collections::HashSet;
use std::vec::Vec;

use crate::reverse_proxy::cookie::read_cookies;

#[derive(Debug, PartialEq, Hash)]
pub struct CookieReplacement {
    // layout: name... '=' replacement... value...
    //                  ^ isep            ^ ivalue
    // TODO replace with Allocator + allocation error checking
    buf: Vec<u8>,
    isep: usize,
    ivalue: usize,
}

#[derive(Debug)]
pub struct CookieReplacementKey {
    k: CookieReplacement,
}

pub type CookieReplacements = HashSet<CookieReplacementKey>;

impl PartialEq for CookieReplacementKey {
    fn eq(&self, other: &Self) -> bool {
        self.k.name() == other.k.name()
    }
}
impl Eq for CookieReplacementKey {}

impl Hash for CookieReplacementKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.k.name().hash(state);
    }
}

impl Borrow<[u8]> for CookieReplacementKey {
    fn borrow(&self) -> &[u8] {
        self.k.name()
    }
}

impl CookieReplacement {
    fn new(name: &[u8], value: &[u8], replacement: &[u8]) -> Self {
        let isep = name.len();
        let ivalue = isep + 1 + replacement.len();
        let mut buf = Vec::with_capacity(ivalue + value.len());
        buf.extend_from_slice(name);
        buf.push(b'=');
        buf.extend_from_slice(replacement);
        buf.extend_from_slice(value);
        Self { buf, isep, ivalue }
    }

    fn as_key_value_cookie(&self) -> &[u8] {
        &self.buf.as_slice()[0..self.ivalue]
    }

    fn name(&self) -> &[u8] {
        &self.buf.as_slice()[0..self.isep]
    }

    fn replacement(&self) -> &[u8] {
        &self.buf.as_slice()[self.isep + 1..self.ivalue]
    }

    fn value(&self) -> &[u8] {
        &self.buf.as_slice()[self.ivalue..]
    }
}

/// Replaces cookies in a HTTP Cookie header.
pub fn cookie_replacements<'a>(
    buffer: &'a mut Vec<u8>,
    cookies: &'a [u8],
    replacements: &CookieReplacements,
) -> &'a [u8] {
    buffer.clear();

    let mut last_index = 0;
    let mut iter = read_cookies(cookies);
    while let Some((name, value)) = iter.next() {
        if let Some(rep) = replacements.get(name) {
            let start_index = ptr_offset(cookies, name);
            buffer.extend_from_slice(&cookies[last_index..start_index]);
            // skip cookie when value don't match
            if value == rep.k.value() {
                buffer.extend_from_slice(rep.k.as_key_value_cookie());
                last_index = ptr_offset(cookies, value) + value.len();
            } else {
                last_index = ptr_offset(cookies, iter.remaining());
            }
        }
    }

    if last_index == 0 {
        return cookies;
    }

    buffer.extend_from_slice(&cookies[last_index..]);
    buffer
}

fn ptr_offset(start: &[u8], pos: &[u8]) -> usize {
    let start = start.as_ptr();
    let pos = pos.as_ptr();
    debug_assert!(start <= pos);
    (unsafe { pos.offset_from(start) }) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_replacements() {
        let mut buffer = Vec::new();
        let mut replacements = HashSet::new();
        replacements.insert(CookieReplacementKey {
            k: CookieReplacement::new(b"name2", b"value2", b"newvalue2!"),
        });
        replacements.insert(CookieReplacementKey {
            k: CookieReplacement::new(b"name4", b"value4", b"newvalue4!"),
        });

        {
            let ref rep = replacements.get(b"name2" as &[u8]).unwrap().k;
            assert_eq!(rep.name(), b"name2");
            assert_eq!(rep.value(), b"value2");
            assert_eq!(rep.replacement(), b"newvalue2!");
        }

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name3=value3",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name3=value3"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name3=value3",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name3=value3"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name3=value3;name2=value2;",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name3=value3;name2=newvalue2!;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name3=value3;name4=value4;",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name3=value3;name4=newvalue4!;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name2=value2; name3=value3;name4=value4;",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name2=newvalue2!; name3=value3;name4=newvalue4!;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name2bis=value2; name3=value3;name4=value4;",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name2bis=value2; name3=value3;name4=newvalue4!;"
        );

        // without ; terminal
        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name2bis=value2; name3=value3;name4=value4",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name2bis=value2; name3=value3;name4=newvalue4!"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=badvalue; name2bis=value2; name3=value3;name4=value4;",
                &replacements
            ))
            .unwrap(),
            "name1=value1;  name2bis=value2; name3=value3;name4=newvalue4!;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name2bis=value2; name3=value3;name4=badvalue;",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name2bis=value2; name3=value3;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(
                &mut buffer,
                b"name1=value1; name2=value2; name2bis=value2; name3=value3;name4=badvalue",
                &replacements
            ))
            .unwrap(),
            "name1=value1; name2=newvalue2!; name2bis=value2; name3=value3;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(&mut buffer,
            b"name1=value1; name2=value2; name2bis=value2; name3=value3;name4=badvalue;name4=badvalue;", &replacements)).unwrap(),
            "name1=value1; name2=newvalue2!; name2bis=value2; name3=value3;"
        );

        assert_eq!(
            std::str::from_utf8(cookie_replacements(&mut buffer,
            b"name1=value1; name2=value2; name2bis=value2; name3=value3;name4=badvalue;name4=badvalue", &replacements)).unwrap(),
            "name1=value1; name2=newvalue2!; name2bis=value2; name3=value3;"
        );
    }
}
