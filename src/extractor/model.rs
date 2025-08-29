use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub url: Url,
    pub title: String,
    pub site_name: Option<String>,
    pub byline: Option<String>,
    pub language: Option<String>,
    pub text: String,
    pub html: String,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ReadabilityResult {
    pub title: String,
    pub site_name: Option<String>,
    pub byline: Option<String>,
    pub text: String,
    pub html: String,
}

pub fn normalize_whitespace(text: &str) -> String {
    // First preserve intentional line breaks and normalize spaces
    let text = text.trim();
    
    // Replace multiple spaces/tabs with single space
    let space_regex = regex::Regex::new(r"[ \t]+").unwrap();
    let spaced = space_regex.replace_all(text, " ");
    
    // Convert multiple consecutive newlines to double newlines
    let newline_regex = regex::Regex::new(r"\n\s*\n+").unwrap();
    newline_regex.replace_all(&spaced, "\n\n").to_string()
}
