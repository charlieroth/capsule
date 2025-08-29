const MIN_CONTENT_LENGTH: usize = 250;
const MIN_WORD_COUNT: usize = 50;
const MAX_BOILERPLATE_RATIO: f64 = 0.3;

pub fn should_reject(title: &str, text: &str) -> bool {
    // Reject if content is too short
    if text.chars().count() < MIN_CONTENT_LENGTH {
        return true;
    }

    let word_count = text.split_whitespace().count();

    // Reject if both title is empty and word count is too low
    if title.trim().is_empty() && word_count < MIN_WORD_COUNT {
        return true;
    }

    // Reject if too much boilerplate content
    if has_too_much_boilerplate(text, word_count) {
        return true;
    }

    false
}

fn has_too_much_boilerplate(text: &str, total_words: usize) -> bool {
    let boilerplate_keywords = [
        "cookie",
        "privacy",
        "terms",
        "service",
        "policy",
        "gdpr",
        "consent",
        "accept",
        "decline",
        "manage",
        "preferences",
        "tracking",
        "advertisement",
        "subscribe",
        "newsletter",
        "login",
        "sign up",
        "register",
        "password",
        "forgot",
        "reset",
        "error",
        "404",
        "not found",
        "access denied",
        "loading",
        "please wait",
        "javascript",
        "enable",
        "browser",
        "update",
        "redirect",
        "continue",
        "click here",
        "read more",
        "learn more",
    ];

    let text_lower = text.to_lowercase();
    let mut boilerplate_count = 0;

    for keyword in &boilerplate_keywords {
        // Count occurrences of each keyword
        let count = text_lower.matches(keyword).count();
        boilerplate_count += count;
    }

    let boilerplate_ratio = boilerplate_count as f64 / total_words as f64;
    boilerplate_ratio > MAX_BOILERPLATE_RATIO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_short_content() {
        assert!(should_reject("Title", "Short"));
        assert!(!should_reject("Title", &"Long enough content ".repeat(50)));
    }

    #[test]
    fn test_reject_empty_title_low_words() {
        let short_text = "Just a few words here";
        assert!(should_reject("", short_text));

        // With a good title, longer text should be accepted
        let longer_text = "This technology discussion provides valuable information about modern software development practices and methodologies that developers can apply to their projects. It includes detailed explanations and practical examples to help readers understand complex concepts in a straightforward manner.";
        assert!(!should_reject("Good Title", longer_text));
    }

    #[test]
    fn test_reject_boilerplate() {
        let boilerplate_text =
            "cookie consent privacy policy terms service gdpr tracking advertisement ".repeat(20);
        assert!(should_reject("Title", &boilerplate_text));
    }

    #[test]
    fn test_accept_good_content() {
        let good_content = "This is a high-quality article with substantial content that provides value to readers. ".repeat(10);
        assert!(!should_reject("Good Article Title", &good_content));
    }
}
