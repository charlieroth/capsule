#![no_main]

use libfuzzer_sys::fuzz_target;
use bytes::Bytes;
use chrono::Utc;
use reqwest::{StatusCode, HeaderMap};
use url::Url;

use capsule::extractor::extract;
use capsule::fetcher::types::{PageResponse, Charset};

fuzz_target!(|data: &[u8]| {
    // Convert raw bytes to string, handling invalid UTF-8 gracefully
    let html = String::from_utf8_lossy(data).to_string();
    
    // Create a test response
    let response = PageResponse {
        url_final: Url::parse("https://example.com").unwrap(),
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body_raw: Bytes::from(html.clone()),
        body_utf8: html,
        charset: Charset::Utf8,
        fetched_at: Utc::now(),
    };
    
    // The extractor should never panic regardless of input
    let _ = futures::executor::block_on(extract(&response));
});
