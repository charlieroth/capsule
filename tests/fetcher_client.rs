use capsule::fetcher::{FetchError, fetch};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn test_fetch_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(
                    "<html><head><title>Test</title></head><body>Hello World</body></html>"
                        .as_bytes(),
                )
                .insert_header("Content-Type", "text/html; charset=utf-8"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());
    let result = fetch(&url).await.unwrap();

    assert!(result.status.is_success());
    assert!(result.body_utf8.contains("Hello World"));
    assert_eq!(result.url_final.as_str(), url);
}

#[tokio::test]
async fn test_fetch_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/notfound"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let url = format!("{}/notfound", mock_server.uri());
    let result = fetch(&url).await;

    match result {
        Err(FetchError::Http { status, retriable }) => {
            assert_eq!(status.as_u16(), 404);
            assert!(!retriable);
        }
        _ => panic!("Expected HTTP 404 error"),
    }
}

#[tokio::test]
async fn test_fetch_500_retryable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let url = format!("{}/error", mock_server.uri());
    let result = fetch(&url).await;

    match result {
        Err(FetchError::Http { status, retriable }) => {
            assert_eq!(status.as_u16(), 500);
            assert!(retriable);
        }
        _ => panic!("Expected HTTP 500 error"),
    }
}

#[tokio::test]
async fn test_fetch_redirect() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/final"))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes("<html><body>Final page</body></html>".as_bytes())
                .insert_header("Content-Type", "text/html"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/redirect", mock_server.uri());
    let result = fetch(&url).await.unwrap();

    assert!(result.status.is_success());
    assert!(result.body_utf8.contains("Final page"));
    assert!(result.url_final.as_str().ends_with("/final"));
}

#[tokio::test]
async fn test_fetch_gzip_compression() {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let original_content =
        "<html><head><title>Compressed</title></head><body>This content is gzipped!</body></html>";

    // Gzip the content
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original_content.as_bytes()).unwrap();
    let compressed_data = encoder.finish().unwrap();

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/gzipped"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(compressed_data)
                .insert_header("Content-Type", "text/html; charset=utf-8")
                .insert_header("Content-Encoding", "gzip"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/gzipped", mock_server.uri());
    let result = fetch(&url).await.unwrap();

    assert!(result.status.is_success());
    assert!(result.body_utf8.contains("This content is gzipped!"));
}

#[tokio::test]
async fn test_fetch_unsupported_content_type() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/image"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0xFF, 0xD8, 0xFF]) // JPEG header
                .insert_header("Content-Type", "image/jpeg"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/image", mock_server.uri());
    let result = fetch(&url).await;

    match result {
        Err(FetchError::UnsupportedContentType(content_type)) => {
            assert_eq!(content_type, "image/jpeg");
        }
        _ => panic!("Expected UnsupportedContentType error"),
    }
}

#[tokio::test]
async fn test_fetch_body_too_large() {
    let mock_server = MockServer::start().await;

    // Create a large body (6MB > 5MB limit)
    let large_body = "x".repeat(6 * 1024 * 1024);

    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(large_body.as_bytes())
                .insert_header("Content-Type", "text/html")
                .insert_header("Content-Length", &(6 * 1024 * 1024).to_string()),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/large", mock_server.uri());
    let result = fetch(&url).await;

    match result {
        Err(FetchError::BodyTooLarge(size)) => {
            assert_eq!(size, 6 * 1024 * 1024);
        }
        _ => panic!("Expected BodyTooLarge error"),
    }
}

#[tokio::test]
async fn test_fetch_invalid_url() {
    let result = fetch("not-a-valid-url").await;

    match result {
        Err(FetchError::InvalidUrl(_)) => {}
        _ => panic!("Expected InvalidUrl error"),
    }
}

#[tokio::test]
async fn test_error_retry_classification() {
    assert!(!FetchError::InvalidUrl(url::ParseError::EmptyHost).should_retry());
    assert!(!FetchError::BodyTooLarge(1000).should_retry());
    assert!(!FetchError::UnsupportedContentType("image/png".to_string()).should_retry());
    assert!(!FetchError::Charset("Invalid encoding".to_string()).should_retry());

    assert!(FetchError::Dns("DNS failure".to_string()).should_retry());
    assert!(FetchError::ConnectTimeout.should_retry());
    assert!(FetchError::RequestTimeout.should_retry());

    // HTTP errors
    assert!(
        !FetchError::Http {
            status: reqwest::StatusCode::NOT_FOUND,
            retriable: false
        }
        .should_retry()
    );
    assert!(
        FetchError::Http {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            retriable: true
        }
        .should_retry()
    );
}
