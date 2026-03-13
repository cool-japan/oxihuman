// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HTTP cookie store — stores, retrieves, and expires cookies.

/// A single HTTP cookie.
#[derive(Clone, Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub max_age_secs: Option<u64>,
    pub secure: bool,
    pub http_only: bool,
    pub created_at: u64,
}

/// The cookie jar that stores all cookies.
pub struct CookieJar {
    cookies: Vec<Cookie>,
    pub max_cookies: usize,
}

/// Creates a new cookie jar.
pub fn new_cookie_jar(max_cookies: usize) -> CookieJar {
    CookieJar {
        cookies: Vec::new(),
        max_cookies,
    }
}

/// Adds or updates a cookie in the jar.
pub fn set_cookie(jar: &mut CookieJar, cookie: Cookie) -> bool {
    /* replace if same name+domain */
    let key_match = |c: &Cookie| c.name == cookie.name && c.domain == cookie.domain;
    if jar.cookies.iter().any(&key_match) {
        jar.cookies.retain(|c| !key_match(c));
        jar.cookies.push(cookie);
        return true;
    }
    if jar.cookies.len() >= jar.max_cookies {
        return false;
    }
    jar.cookies.push(cookie);
    true
}

/// Retrieves a cookie by name and domain (domain=None matches any).
pub fn get_cookie<'a>(
    jar: &'a CookieJar,
    name: &str,
    domain: Option<&str>,
    now: u64,
) -> Option<&'a Cookie> {
    jar.cookies.iter().find(|c| {
        c.name == name && (domain.is_none() || c.domain.as_deref() == domain) && !is_expired(c, now)
    })
}

/// Removes a cookie by name and domain.
pub fn delete_cookie(jar: &mut CookieJar, name: &str, domain: Option<&str>) -> bool {
    let before = jar.cookies.len();
    jar.cookies
        .retain(|c| !(c.name == name && (domain.is_none() || c.domain.as_deref() == domain)));
    jar.cookies.len() < before
}

/// Removes all expired cookies.
pub fn purge_expired_cookies(jar: &mut CookieJar, now: u64) -> usize {
    let before = jar.cookies.len();
    jar.cookies.retain(|c| !is_expired(c, now));
    before.saturating_sub(jar.cookies.len())
}

fn is_expired(cookie: &Cookie, now: u64) -> bool {
    if let Some(max_age) = cookie.max_age_secs {
        now.saturating_sub(cookie.created_at) >= max_age
    } else {
        false /* session cookie, never expires */
    }
}

/// Serializes a cookie to a Set-Cookie header value string.
pub fn serialize_set_cookie(cookie: &Cookie) -> String {
    let mut s = format!("{}={}", cookie.name, cookie.value);
    if let Some(d) = &cookie.domain {
        s.push_str(&format!("; Domain={}", d));
    }
    if let Some(p) = &cookie.path {
        s.push_str(&format!("; Path={}", p));
    }
    if let Some(ma) = cookie.max_age_secs {
        s.push_str(&format!("; Max-Age={}", ma));
    }
    if cookie.secure {
        s.push_str("; Secure");
    }
    if cookie.http_only {
        s.push_str("; HttpOnly");
    }
    s
}

/// Returns the total number of cookies in the jar.
pub fn jar_size(jar: &CookieJar) -> usize {
    jar.cookies.len()
}

impl CookieJar {
    /// Creates a new cookie jar with given capacity.
    pub fn new(max_cookies: usize) -> Self {
        new_cookie_jar(max_cookies)
    }
}

fn make_cookie(name: &str, value: &str, max_age: Option<u64>, created_at: u64) -> Cookie {
    Cookie {
        name: name.into(),
        value: value.into(),
        domain: None,
        path: None,
        max_age_secs: max_age,
        secure: false,
        http_only: false,
        created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jar() -> CookieJar {
        new_cookie_jar(32)
    }

    #[test]
    fn test_set_and_get_cookie() {
        let mut jar = make_jar();
        set_cookie(&mut jar, make_cookie("sid", "abc", None, 0));
        let c = get_cookie(&jar, "sid", None, 0);
        assert!(c.is_some());
        assert_eq!(c.expect("should succeed").value, "abc");
    }

    #[test]
    fn test_update_existing_cookie() {
        let mut jar = make_jar();
        set_cookie(&mut jar, make_cookie("sid", "v1", None, 0));
        set_cookie(&mut jar, make_cookie("sid", "v2", None, 0));
        assert_eq!(jar_size(&jar), 1);
        assert_eq!(
            get_cookie(&jar, "sid", None, 0)
                .expect("should succeed")
                .value,
            "v2"
        );
    }

    #[test]
    fn test_expired_cookie_not_returned() {
        let mut jar = make_jar();
        set_cookie(&mut jar, make_cookie("tok", "val", Some(30), 0));
        /* now=100 => age=100 >= max_age=30 => expired */
        assert!(get_cookie(&jar, "tok", None, 100).is_none());
    }

    #[test]
    fn test_delete_cookie() {
        let mut jar = make_jar();
        set_cookie(&mut jar, make_cookie("x", "1", None, 0));
        assert!(delete_cookie(&mut jar, "x", None));
        assert_eq!(jar_size(&jar), 0);
    }

    #[test]
    fn test_delete_nonexistent_returns_false() {
        let mut jar = make_jar();
        assert!(!delete_cookie(&mut jar, "none", None));
    }

    #[test]
    fn test_purge_expired_removes_old_cookies() {
        let mut jar = make_jar();
        set_cookie(&mut jar, make_cookie("a", "1", Some(10), 0)); /* expires at 10 */
        set_cookie(&mut jar, make_cookie("b", "2", Some(1000), 0)); /* expires at 1000 */
        let removed = purge_expired_cookies(&mut jar, 100);
        assert_eq!(removed, 1);
        assert_eq!(jar_size(&jar), 1);
    }

    #[test]
    fn test_serialize_set_cookie_basic() {
        let c = make_cookie("sid", "abc123", Some(3600), 0);
        let s = serialize_set_cookie(&c);
        assert!(s.contains("sid=abc123"));
        assert!(s.contains("Max-Age=3600"));
    }

    #[test]
    fn test_capacity_limit() {
        let mut jar = new_cookie_jar(2);
        set_cookie(&mut jar, make_cookie("a", "1", None, 0));
        set_cookie(&mut jar, make_cookie("b", "2", None, 0));
        let ok = set_cookie(&mut jar, make_cookie("c", "3", None, 0));
        assert!(!ok);
    }

    #[test]
    fn test_session_cookie_does_not_expire() {
        let mut jar = make_jar();
        set_cookie(&mut jar, make_cookie("sess", "xyz", None, 0)); /* no max_age */
        /* even after long time, session cookie lives */
        assert!(get_cookie(&jar, "sess", None, 99999).is_some());
    }
}
