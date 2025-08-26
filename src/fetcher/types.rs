use bytes::Bytes;
use chrono::{DateTime, Utc};
use reqwest::{StatusCode, header::HeaderMap};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Charset {
    Utf8,
    Latin1,
    Windows1252,
    Iso88591,
    ShiftJis,
    Gb2312,
    Big5,
    Other(String),
}

impl Charset {
    pub fn from_encoding(encoding: &encoding_rs::Encoding) -> Self {
        use std::ptr;

        if ptr::eq(encoding, encoding_rs::UTF_8) {
            Self::Utf8
        } else if ptr::eq(encoding, encoding_rs::WINDOWS_1252) {
            Self::Windows1252
        } else if ptr::eq(encoding, encoding_rs::SHIFT_JIS) {
            Self::ShiftJis
        } else if ptr::eq(encoding, encoding_rs::GBK) || ptr::eq(encoding, encoding_rs::GB18030) {
            Self::Gb2312
        } else if ptr::eq(encoding, encoding_rs::BIG5) {
            Self::Big5
        } else {
            // For other encodings, assume Latin1 for most cases or Other
            // This is a simplified approach to avoid lifetime issues
            Self::Other("unknown".to_string())
        }
    }
}

#[derive(Debug)]
pub struct PageResponse {
    pub url_final: Url,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body_raw: Bytes,
    pub body_utf8: String,
    pub charset: Charset,
    pub fetched_at: DateTime<Utc>,
}
