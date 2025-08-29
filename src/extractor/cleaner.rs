use ammonia::Builder;
use regex::Regex;
use url::Url;

use crate::extractor::model::{ReadabilityResult, normalize_whitespace};

pub fn sanitize_and_resolve_links(result: &mut ReadabilityResult, base_url: &Url) {
    // Clean the HTML with Ammonia (removes scripts, styles, dangerous elements)
    let clean_html = Builder::default().clean(&result.html).to_string();

    // Manually resolve relative links to absolute
    result.html = resolve_links(&clean_html, base_url);

    // Normalize whitespace in text content
    result.text = normalize_whitespace(&result.text);
}

fn resolve_links(html: &str, base_url: &Url) -> String {
    // Resolve relative href attributes
    let href_regex = Regex::new(r#"href="([^"]+)""#).unwrap();
    let html = href_regex.replace_all(html, |caps: &regex::Captures| {
        let url_str = &caps[1];
        if let Ok(absolute_url) = base_url.join(url_str) {
            format!(r#"href="{}""#, absolute_url)
        } else {
            caps[0].to_string()
        }
    });

    // Resolve relative src attributes
    let src_regex = Regex::new(r#"src="([^"]+)""#).unwrap();
    let html = src_regex.replace_all(&html, |caps: &regex::Captures| {
        let url_str = &caps[1];
        if let Ok(absolute_url) = base_url.join(url_str) {
            format!(r#"src="{}""#, absolute_url)
        } else {
            caps[0].to_string()
        }
    });

    html.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_removes_dangerous_elements() {
        let mut result = ReadabilityResult {
            title: "Test".to_string(),
            site_name: None,
            byline: None,
            text: "Hello world".to_string(),
            html:
                r#"<p>Hello world</p><script>alert('xss')</script><style>body{color:red}</style>"#
                    .to_string(),
        };

        let base_url = Url::parse("https://example.com").unwrap();
        sanitize_and_resolve_links(&mut result, &base_url);

        assert!(!result.html.contains("<script"));
        assert!(!result.html.contains("<style"));
        assert!(result.html.contains("<p>Hello world</p>"));
    }

    #[test]
    fn test_resolve_relative_links() {
        let mut result = ReadabilityResult {
            title: "Test".to_string(),
            site_name: None,
            byline: None,
            text: "Click here".to_string(),
            html: r#"<p><a href="/page">Click here</a></p><img src="image.jpg" alt="test">"#
                .to_string(),
        };

        let base_url = Url::parse("https://example.com/article/").unwrap();
        sanitize_and_resolve_links(&mut result, &base_url);

        assert!(result.html.contains("https://example.com/page"));
        assert!(
            result
                .html
                .contains("https://example.com/article/image.jpg")
        );
    }

    #[test]
    fn test_normalize_whitespace() {
        let text = "  Hello    world  \n\n\n  Test  ";
        let normalized = normalize_whitespace(text);
        // The function preserves newlines and normalizes spaces
        assert_eq!(normalized, "Hello world \n\n Test");
    }
}
