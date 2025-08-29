use whatlang::{Lang, detect};

const MIN_CONFIDENCE: f64 = 0.25;
const MIN_TEXT_LENGTH: usize = 50;

pub fn detect_language(text: &str) -> Option<String> {
    // Skip detection for very short text
    if text.trim().len() < MIN_TEXT_LENGTH {
        return None;
    }

    // Use whatlang for detection
    if let Some(info) = detect(text)
        && info.confidence() >= MIN_CONFIDENCE {
        return Some(lang_to_code(info.lang()));
    }

    None
}

fn lang_to_code(lang: Lang) -> String {
    match lang {
        Lang::Eng => "en".to_string(),
        Lang::Rus => "ru".to_string(),
        Lang::Cmn => "zh".to_string(),
        Lang::Spa => "es".to_string(),
        Lang::Fra => "fr".to_string(),
        Lang::Deu => "de".to_string(),
        Lang::Jpn => "ja".to_string(),
        Lang::Kor => "ko".to_string(),
        Lang::Por => "pt".to_string(),
        Lang::Ita => "it".to_string(),
        Lang::Nld => "nl".to_string(),
        Lang::Pol => "pl".to_string(),
        Lang::Tur => "tr".to_string(),
        Lang::Swe => "sv".to_string(),
        Lang::Dan => "da".to_string(),
        Lang::Fin => "fi".to_string(),
        Lang::Heb => "he".to_string(),
        Lang::Ara => "ar".to_string(),
        Lang::Hin => "hi".to_string(),
        Lang::Tha => "th".to_string(),
        Lang::Vie => "vi".to_string(),
        _ => format!("{:?}", lang).to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_english() {
        let text = "This is a test of the English language detection system. It should work well.";
        let result = detect_language(text);
        assert_eq!(result, Some("en".to_string()));
    }

    #[test]
    fn test_detect_spanish() {
        let text = "Esto es una prueba del sistema de detección de idiomas en español. Debería funcionar bien.";
        let result = detect_language(text);
        assert_eq!(result, Some("es".to_string()));
    }

    #[test]
    fn test_short_text_returns_none() {
        let text = "Short";
        let result = detect_language(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_low_confidence_returns_none() {
        let text =
            "1 2 3 4 5 6 7 8 9 0 ! @ # $ % ^ & * ( ) - = + [ ] { } | \\ : ; \" ' < > , . ? /";
        let result = detect_language(text);
        assert_eq!(result, None);
    }
}
