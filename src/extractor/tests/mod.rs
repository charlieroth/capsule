use bytes::Bytes;
use chrono::Utc;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use std::fs;
use url::Url;

use crate::extractor::extract;
use crate::fetcher::types::{Charset, PageResponse};

#[tokio::test]
async fn test_extract_article() {
    let html = fs::read_to_string("src/extractor/tests/fixtures/article.html")
        .expect("Failed to read test fixture");

    let response = create_test_response(html, "https://example.com/article");
    let result = extract(&response).await;

    assert!(result.is_some());
    let content = result.unwrap();

    // The title might include the site name depending on the HTML structure
    assert!(content.title.contains("Sample Article"));
    assert_eq!(content.site_name, Some("News Site".to_string()));
    assert!(content.text.contains("first paragraph"));
    assert!(content.text.contains("second paragraph"));
    assert!(!content.html.contains("<script"));
    assert!(!content.html.contains("<style"));
    assert!(!content.html.contains("<nav"));

    // Check that relative links are resolved
    assert!(content.html.contains("https://example.com/related"));
    assert!(
        content
            .html
            .contains("https://example.com/images/sample.jpg")
    );
}

#[tokio::test]
async fn test_extract_blog_post() {
    let html = fs::read_to_string("src/extractor/tests/fixtures/blog.html")
        .expect("Failed to read test fixture");

    let response = create_test_response(html, "https://blog.example.com/post");
    let result = extract(&response).await;

    assert!(result.is_some());
    let content = result.unwrap();

    // The title might include the site name depending on the HTML structure
    assert!(content.title.contains("How to Build Better Software"));
    assert_eq!(content.site_name, Some("Tech Blog".to_string()));
    assert!(content.text.contains("Building better software"));
    assert!(content.text.contains("Key Principles"));
    assert_eq!(content.language, Some("en".to_string()));
}

#[tokio::test]
async fn test_reject_empty_page() {
    let html = fs::read_to_string("src/extractor/tests/fixtures/empty.html")
        .expect("Failed to read test fixture");

    let response = create_test_response(html, "https://example.com/empty");
    let result = extract(&response).await;

    // Should be rejected due to boilerplate content and insufficient text
    assert!(result.is_none());
}

#[tokio::test]
async fn test_minimal_valid_content() {
    let html = format!(
        r#"<!DOCTYPE html><html><head><title>Valid Article</title></head><body><article><h1>Valid Article</h1><p>{}</p></article></body></html>"#,
        "This is a valid article with enough content to pass the minimum requirements for extraction. ".repeat(20)
    );

    let response = create_test_response(html, "https://example.com/valid");
    let result = extract(&response).await;

    assert!(result.is_some());
    let content = result.unwrap();
    assert_eq!(content.title, "Valid Article");
    assert!(content.text.len() > 250);
}

#[tokio::test]
async fn test_malformed_html() {
    let html =
        "<html><head><title>Broken</title><body><p>Unclosed tags<div>More content".to_string();

    let response = create_test_response(html, "https://example.com/broken");
    let result = extract(&response).await;

    // Should handle malformed HTML gracefully
    if let Some(content) = result {
        assert_eq!(content.title, "Broken");
        assert!(content.text.contains("Unclosed tags"));
    }
}

fn create_test_response(html: String, url: &str) -> PageResponse {
    PageResponse {
        url_final: Url::parse(url).unwrap(),
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body_raw: Bytes::from(html.clone()),
        body_utf8: html,
        charset: Charset::Utf8,
        fetched_at: Utc::now(),
    }
}

#[cfg(feature = "fuzz")]
mod fuzz {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_extract_never_panics(
            html in ".*",
            url in "https://[a-z]+\\.com/.*"
        ) {
            let response = create_test_response(html, &url);
            // Should never panic regardless of input
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(extract(&response));
        }

        #[test]
        fn test_extract_valid_utf8_output(
            html in ".*",
        ) {
            let response = create_test_response(html, "https://example.com");
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Some(content) = rt.block_on(extract(&response)) {
                // All output should be valid UTF-8
                assert!(content.text.is_ascii() || content.text.chars().all(|c| !c.is_control() || c == '\n'));
                assert!(content.html.is_ascii() || content.html.chars().all(|c| !c.is_control() || c == '\n' || c == '\t'));
            }
        }
    }
}
