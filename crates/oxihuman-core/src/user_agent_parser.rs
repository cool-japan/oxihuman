// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! User-Agent string parser stub — extracts browser, OS, and version info.

/// Browser family detected from a User-Agent string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowserFamily {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Opera,
    InternetExplorer,
    Bot,
    Unknown,
}

/// Operating system detected from a User-Agent string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OsFamily {
    Windows,
    MacOs,
    Linux,
    Android,
    Ios,
    Unknown,
}

/// Parsed result of a User-Agent string.
#[derive(Clone, Debug)]
pub struct UserAgent {
    pub raw: String,
    pub browser: BrowserFamily,
    pub os: OsFamily,
    pub version: Option<String>,
    pub is_mobile: bool,
    pub is_bot: bool,
}

/// Parses a User-Agent string into structured information.
pub fn parse_user_agent(ua: &str) -> UserAgent {
    let lower = ua.to_lowercase();
    let is_bot = is_bot_agent(&lower);
    let browser = detect_browser(&lower);
    let os = detect_os(&lower);
    let is_mobile =
        lower.contains("mobile") || lower.contains("android") || lower.contains("iphone");
    let version = extract_version(ua);

    UserAgent {
        raw: ua.into(),
        browser,
        os,
        version,
        is_mobile,
        is_bot,
    }
}

fn is_bot_agent(lower: &str) -> bool {
    lower.contains("bot")
        || lower.contains("crawler")
        || lower.contains("spider")
        || lower.contains("slurp")
        || lower.contains("googlebot")
}

fn detect_browser(lower: &str) -> BrowserFamily {
    if lower.contains("edg/") || lower.contains("edge/") {
        BrowserFamily::Edge
    } else if lower.contains("opr/") || lower.contains("opera") {
        BrowserFamily::Opera
    } else if lower.contains("firefox") {
        BrowserFamily::Firefox
    } else if lower.contains("chrome") {
        BrowserFamily::Chrome
    } else if lower.contains("safari") {
        BrowserFamily::Safari
    } else if lower.contains("msie") || lower.contains("trident") {
        BrowserFamily::InternetExplorer
    } else if lower.contains("bot") || lower.contains("crawler") {
        BrowserFamily::Bot
    } else {
        BrowserFamily::Unknown
    }
}

fn detect_os(lower: &str) -> OsFamily {
    if lower.contains("android") {
        OsFamily::Android
    } else if lower.contains("iphone") || lower.contains("ipad") {
        OsFamily::Ios
    } else if lower.contains("windows") {
        OsFamily::Windows
    } else if lower.contains("mac os") || lower.contains("macos") {
        OsFamily::MacOs
    } else if lower.contains("linux") {
        OsFamily::Linux
    } else {
        OsFamily::Unknown
    }
}

fn extract_version(ua: &str) -> Option<String> {
    /* attempt to extract version from "Version/X.Y" or "Firefox/X.Y" patterns */
    for prefix in &["Version/", "Firefox/", "Chrome/", "OPR/", "Edg/"] {
        if let Some(pos) = ua.find(prefix) {
            let rest = &ua[pos + prefix.len()..];
            let end = rest
                .find(|c: char| !c.is_ascii_digit() && c != '.')
                .unwrap_or(rest.len());
            if end > 0 {
                return Some(rest[..end].into());
            }
        }
    }
    None
}

/// Returns a display name string for the browser family.
pub fn browser_name(family: &BrowserFamily) -> &'static str {
    match family {
        BrowserFamily::Chrome => "Chrome",
        BrowserFamily::Firefox => "Firefox",
        BrowserFamily::Safari => "Safari",
        BrowserFamily::Edge => "Edge",
        BrowserFamily::Opera => "Opera",
        BrowserFamily::InternetExplorer => "Internet Explorer",
        BrowserFamily::Bot => "Bot",
        BrowserFamily::Unknown => "Unknown",
    }
}

/// Returns a display name for the OS family.
pub fn os_name(family: &OsFamily) -> &'static str {
    match family {
        OsFamily::Windows => "Windows",
        OsFamily::MacOs => "macOS",
        OsFamily::Linux => "Linux",
        OsFamily::Android => "Android",
        OsFamily::Ios => "iOS",
        OsFamily::Unknown => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_firefox() {
        let ua = parse_user_agent(
            "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0",
        );
        assert_eq!(ua.browser, BrowserFamily::Firefox);
    }

    #[test]
    fn test_detect_chrome() {
        let ua = parse_user_agent("Mozilla/5.0 Chrome/112.0.0.0 Safari/537.36");
        assert_eq!(ua.browser, BrowserFamily::Chrome);
    }

    #[test]
    fn test_detect_edge() {
        let ua = parse_user_agent("Mozilla/5.0 Edg/112.0.1722.58");
        assert_eq!(ua.browser, BrowserFamily::Edge);
    }

    #[test]
    fn test_detect_windows() {
        let ua = parse_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)");
        assert_eq!(ua.os, OsFamily::Windows);
    }

    #[test]
    fn test_detect_linux() {
        let ua = parse_user_agent("Mozilla/5.0 (X11; Linux x86_64)");
        assert_eq!(ua.os, OsFamily::Linux);
    }

    #[test]
    fn test_detect_android_mobile() {
        let ua = parse_user_agent("Mozilla/5.0 (Linux; Android 13) Mobile");
        assert_eq!(ua.os, OsFamily::Android);
        assert!(ua.is_mobile);
    }

    #[test]
    fn test_detect_bot() {
        let ua = parse_user_agent("Googlebot/2.1 (+http://www.google.com/bot.html)");
        assert!(ua.is_bot);
        assert_eq!(ua.browser, BrowserFamily::Bot);
    }

    #[test]
    fn test_extract_firefox_version() {
        let ua = parse_user_agent("Mozilla/5.0 Firefox/109.0");
        assert_eq!(ua.version.as_deref(), Some("109.0"));
    }

    #[test]
    fn test_browser_name_display() {
        assert_eq!(browser_name(&BrowserFamily::Safari), "Safari");
        assert_eq!(os_name(&OsFamily::Ios), "iOS");
    }

    #[test]
    fn test_unknown_user_agent() {
        let ua = parse_user_agent("CustomClient/1.0");
        assert_eq!(ua.browser, BrowserFamily::Unknown);
        assert_eq!(ua.os, OsFamily::Unknown);
        assert!(!ua.is_mobile);
        assert!(!ua.is_bot);
    }
}
