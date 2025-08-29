pub mod cleaner;
pub mod language;
pub mod model;
pub mod reader;
pub mod reject;

#[cfg(test)]
mod tests;

pub use model::ExtractedContent;

use crate::fetcher::types::PageResponse;

pub async fn extract(resp: &PageResponse) -> Option<ExtractedContent> {
    // 1. Extract readable content using readability
    let mut result = reader::extract(&resp.body_utf8, resp.url_final.clone())?;

    // 2. Clean and sanitize HTML, resolve links
    cleaner::sanitize_and_resolve_links(&mut result, &resp.url_final);

    // 3. Detect language
    let detected_language = language::detect_language(&result.text);

    // 4. Check if content should be rejected
    if reject::should_reject(&result.title, &result.text) {
        return None;
    }

    // 5. Create final extracted content
    Some(ExtractedContent {
        url: resp.url_final.clone(),
        title: result.title,
        site_name: result.site_name,
        byline: result.byline,
        language: detected_language,
        text: result.text,
        html: result.html,
        fetched_at: resp.fetched_at,
    })
}
