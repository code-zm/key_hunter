use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// OpenAI API keys start with "sk-" followed by 48 alphanumeric characters
    static ref OPENAI_PATTERN: Regex = Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap();
}

pub struct OpenAIDetector {
    patterns: Vec<Regex>,
}

impl OpenAIDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![OPENAI_PATTERN.clone()],
        }
    }
}

impl Default for OpenAIDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for OpenAIDetector {
    fn name(&self) -> &str {
        "openai"
    }

    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.find_iter(content) {
                let key = capture.as_str();
                let (line_number, context) =
                    PatternUtils::get_line_context(content, capture.start(), 2);

                detected.push(DetectedKey {
                    key: key.to_string(),
                    key_type: "openai".to_string(),
                    repository: String::new(), // Filled in by search provider
                    file_path: file_path.to_string(),
                    file_url: String::new(), // Filled in by search provider
                    line_number: Some(line_number),
                    context: Some(context),
                });
            }
        }

        detected
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yml", ".yaml", ".sh", ".go", ".rs", ".ts", ".txt"]
    }

    fn search_queries(&self) -> Vec<String> {
        vec![
            "OPENAI_API_KEY".to_string(),
            "sk- AND openai".to_string(),
            "openai AND api_key extension:env".to_string(),
            "openai AND api_key extension:py".to_string(),
            "openai AND api_key extension:json".to_string(),
            "openai AND api_key extension:js".to_string(),
            "openai AND api_key extension:ts".to_string(),
            "\"sk-\" extension:env".to_string(),
            "OPENAI_KEY".to_string(),
        ]
    }

    fn filter_key(&self, key: &str) -> bool {
        // OpenAI keys must start with sk- and be exactly 51 characters (sk- + 48)
        key.starts_with("sk-") && key.len() == 51
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_detector_basic() {
        let detector = OpenAIDetector::new();
        assert_eq!(detector.name(), "openai");
    }

    #[test]
    fn test_detect_valid_key() {
        let detector = OpenAIDetector::new();
        // Valid pattern: sk- (3) + 48 alphanumeric chars = 51 total
        let content = "OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKL";

        let results = detector.detect(content, "test.env");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "sk-abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKL");
    }

    #[test]
    fn test_filter_wrong_length() {
        let detector = OpenAIDetector::new();
        // Wrong length - should not match
        let content = "key=sk-tooshort";

        let results = detector.detect(content, "test.txt");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_no_prefix() {
        let detector = OpenAIDetector::new();
        // No sk- prefix
        let content = "key=abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJ";

        let results = detector.detect(content, "test.txt");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_queries() {
        let detector = OpenAIDetector::new();
        let queries = detector.search_queries();
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.contains("OPENAI_API_KEY")));
    }
}
