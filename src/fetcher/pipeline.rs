use crate::fetcher::{
    errors::FetchError,
    types::{Charset, PageResponse},
};
use bytes::Bytes;
use chrono::Utc;
use encoding_rs::Encoding;
use regex::Regex;
use reqwest::{StatusCode, header::HeaderMap};
use std::sync::LazyLock;
use url::Url;

static CHARSET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)charset\s*=\s*["']?([^"'\s;]+)"#).unwrap());

static META_CHARSET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)<meta\s+[^>]*?charset\s*=\s*["']?([^"'\s/>]+)"#).unwrap());

static META_HTTP_EQUIV_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+[^>]*?http-equiv\s*=\s*["']?content-type["']?[^>]*?content\s*=\s*["']?[^"'>]*?charset\s*=\s*([^"'\s;/>]+)"#).unwrap()
});

pub fn process_response(
    url_final: Url,
    status: StatusCode,
    headers: HeaderMap,
    body_bytes: Bytes,
    content_type: &str,
) -> Result<PageResponse, FetchError> {
    let charset = detect_charset(content_type, &body_bytes)?;
    let body_utf8 = decode_to_utf8(&body_bytes, &charset)?;

    Ok(PageResponse {
        url_final,
        status,
        headers,
        body_raw: body_bytes,
        body_utf8,
        charset,
        fetched_at: Utc::now(),
    })
}

fn detect_charset(content_type: &str, body_bytes: &[u8]) -> Result<Charset, FetchError> {
    // 1. Check Content-Type header for charset
    if let Some(captures) = CHARSET_REGEX.captures(content_type) {
        if let Some(charset_str) = captures.get(1) {
            let charset_name = charset_str.as_str().to_lowercase();
            if let Some(encoding) = Encoding::for_label(charset_name.as_bytes()) {
                return Ok(Charset::from_encoding(encoding));
            }
        }
    }

    // 2. Check for <meta charset> in first 4KB
    let search_bytes = &body_bytes[..body_bytes.len().min(4096)];
    let search_str = String::from_utf8_lossy(search_bytes);

    // Look for <meta charset="...">
    if let Some(captures) = META_CHARSET_REGEX.captures(&search_str) {
        if let Some(charset_str) = captures.get(1) {
            let charset_name = charset_str.as_str().to_lowercase();
            if let Some(encoding) = Encoding::for_label(charset_name.as_bytes()) {
                return Ok(Charset::from_encoding(encoding));
            }
        }
    }

    // Look for <meta http-equiv="Content-Type" content="...; charset=...">
    if let Some(captures) = META_HTTP_EQUIV_REGEX.captures(&search_str) {
        if let Some(charset_str) = captures.get(1) {
            let charset_name = charset_str.as_str().to_lowercase();
            if let Some(encoding) = Encoding::for_label(charset_name.as_bytes()) {
                return Ok(Charset::from_encoding(encoding));
            }
        }
    }

    // 3. Use chardet for heuristic detection
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(search_bytes, false);
    let detected = detector.guess(None, true);

    Ok(Charset::from_encoding(detected))
}

fn decode_to_utf8(body_bytes: &[u8], charset: &Charset) -> Result<String, FetchError> {
    let encoding = match charset {
        Charset::Utf8 => encoding_rs::UTF_8,
        Charset::Latin1 | Charset::Iso88591 => encoding_rs::WINDOWS_1252,
        Charset::Windows1252 => encoding_rs::WINDOWS_1252,
        Charset::ShiftJis => encoding_rs::SHIFT_JIS,
        Charset::Gb2312 => encoding_rs::GBK,
        Charset::Big5 => encoding_rs::BIG5,
        Charset::Other(name) => Encoding::for_label(name.as_bytes()).unwrap_or(encoding_rs::UTF_8),
    };

    let (decoded, _encoding, had_errors) = encoding.decode(body_bytes);

    if had_errors {
        return Err(FetchError::Charset(format!(
            "Failed to decode content with encoding: {}",
            encoding.name()
        )));
    }

    Ok(decoded.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_charset_from_content_type() {
        let content_type = "text/html; charset=utf-8";
        let body = b"<html><head><title>Test</title></head></html>";

        let charset = detect_charset(content_type, body).unwrap();
        assert!(matches!(charset, Charset::Utf8));
    }

    #[test]
    fn test_detect_charset_from_meta_tag() {
        let content_type = "text/html";
        let body = b"<html><head><meta charset=\"iso-8859-1\"><title>Test</title></head></html>";

        let charset = detect_charset(content_type, body).unwrap();
        // ISO-8859-1 gets mapped to Windows1252 by encoding_rs since it's a superset
        assert!(matches!(charset, Charset::Windows1252));
    }

    #[test]
    fn test_detect_charset_from_meta_http_equiv() {
        let content_type = "text/html";
        let body = b"<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=windows-1252\"><title>Test</title></head></html>";

        let charset = detect_charset(content_type, body).unwrap();
        assert!(matches!(charset, Charset::Windows1252));
    }

    #[test]
    fn test_decode_utf8() {
        let body = "Hello, 世界!".as_bytes();
        let charset = Charset::Utf8;

        let decoded = decode_to_utf8(body, &charset).unwrap();
        assert_eq!(decoded, "Hello, 世界!");
    }
}
