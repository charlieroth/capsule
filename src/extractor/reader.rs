use readability::extractor;
use scraper::{Html, Selector};
use url::Url;

use crate::extractor::model::ReadabilityResult;

pub fn extract(html: &str, url: Url) -> Option<ReadabilityResult> {
    // Try readability first
    if let Ok(article) = extractor::extract(&mut html.as_bytes(), &url) {
        return Some(ReadabilityResult {
            title: article.title,
            site_name: extract_site_name(html),
            byline: None, // readability crate doesn't provide byline
            text: article.text,
            html: article.content,
        });
    }

    // Fallback to basic scraping if readability fails
    fallback_extract(html)
}

fn extract_site_name(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    
    // Try og:site_name first
    let selector = Selector::parse("meta[property='og:site_name']").ok()?;
    if let Some(element) = document.select(&selector).next()
        && let Some(content) = element.value().attr("content") {
        return Some(content.to_string());
    }
    
    // Try site name from title
    let title_selector = Selector::parse("title").ok()?;
    if let Some(element) = document.select(&title_selector).next() {
        let title = element.text().collect::<String>();
        // Look for patterns like "Article Title - Site Name" or "Article Title | Site Name"
        if let Some(pos) = title.rfind(" - ") {
            return Some(title[pos + 3..].to_string());
        }
        if let Some(pos) = title.rfind(" | ") {
            return Some(title[pos + 3..].to_string());
        }
    }
    
    None
}

fn fallback_extract(html: &str) -> Option<ReadabilityResult> {
    let document = Html::parse_document(html);
    
    // Extract title
    let title = extract_title(&document)?;
    
    // Extract main content using basic heuristics
    let (text, html_content) = extract_main_content(&document);
    
    if text.trim().is_empty() {
        return None;
    }
    
    Some(ReadabilityResult {
        title,
        site_name: extract_site_name(html),
        byline: None,
        text,
        html: html_content,
    })
}

fn extract_title(document: &Html) -> Option<String> {
    // Try og:title first
    if let Ok(selector) = Selector::parse("meta[property='og:title']") {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                return Some(content.to_string());
            }
        }
    }
    
    // Try regular title
    if let Ok(selector) = Selector::parse("title") {
        for element in document.select(&selector) {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }
    
    // Try h1
    if let Ok(selector) = Selector::parse("h1") {
        for element in document.select(&selector) {
            let title = element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }
    
    None
}

fn extract_main_content(document: &Html) -> (String, String) {
    let content_selectors = vec![
        "article",
        "main", 
        "[role='main']",
        ".content",
        ".post",
        ".article",
        "#content",
        "#main",
        ".entry-content",
    ];
    
    for selector_str in content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>();
                let html = element.html();
                if text.trim().len() > 100 { // Basic length check
                    return (text, html);
                }
            }
        }
    }
    
    // Last resort: try body but exclude common boilerplate elements
    if let Ok(body_selector) = Selector::parse("body")
        && let Some(body) = document.select(&body_selector).next() {
        let text = body.text().collect::<String>();
        let html = body.html();
        return (text, html);
    }
    
    (String::new(), String::new())
}
