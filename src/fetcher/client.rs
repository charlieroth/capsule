use crate::fetcher::{errors::FetchError, pipeline::process_response, types::PageResponse};
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use tracing::instrument;

const MAX_BODY_SIZE: u64 = 5 * 1024 * 1024; // 5MB
const USER_AGENT: &str = "CapsuleBot/0.1 (+https://capsule.example.com)";

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::limited(10))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                    .parse()
                    .unwrap(),
            );
            headers
        })
        .build()
        .expect("Failed to build HTTP client")
});

pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}

#[instrument(skip_all, fields(url = %url))]
pub async fn fetch(url: &str) -> Result<PageResponse, FetchError> {
    let parsed_url = url::Url::parse(url)?;

    let response = HTTP_CLIENT
        .get(parsed_url.clone())
        .send()
        .await
        .map_err(FetchError::from_reqwest_error)?;

    // Check content length before downloading
    if let Some(content_length) = response.content_length()
        && content_length > MAX_BODY_SIZE
    {
        return Err(FetchError::BodyTooLarge(content_length));
    }

    let final_url = response.url().clone();
    let status = response.status();
    let headers = response.headers().clone();

    // Check if we got a successful response
    if !status.is_success() {
        return Err(FetchError::Http {
            status,
            retriable: status.is_server_error(),
        });
    }

    // Get content type
    let content_type = headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("text/html")
        .to_string();

    // Only process HTML content for now
    if !content_type.contains("text/html") && !content_type.contains("application/xhtml") {
        return Err(FetchError::UnsupportedContentType(content_type.clone()));
    }

    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| FetchError::Io(e.to_string()))?;

    // Check body size after download (in case Content-Length was missing)
    if body_bytes.len() as u64 > MAX_BODY_SIZE {
        return Err(FetchError::BodyTooLarge(body_bytes.len() as u64));
    }

    process_response(final_url, status, headers, body_bytes, &content_type)
}
