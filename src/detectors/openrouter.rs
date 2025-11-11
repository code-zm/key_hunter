use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// OpenRouter API keys start with "sk-or-v1-" followed by 64 hex characters
    static ref OPENROUTER_PATTERN: Regex = Regex::new(r"sk-or-v1-[a-f0-9]{64}").unwrap();
}

pub struct OpenRouterDetector {
    patterns: Vec<Regex>,
}

impl OpenRouterDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![OPENROUTER_PATTERN.clone()],
        }
    }
}

impl Default for OpenRouterDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for OpenRouterDetector {
    fn name(&self) -> &str {
        "openrouter"
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
                    key_type: "openrouter".to_string(),
                    repository: String::new(),
                    file_path: file_path.to_string(),
                    file_url: String::new(),
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
            "OPENROUTER_API_KEY".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_detector_basic() {
        let detector = OpenRouterDetector::new();
        assert_eq!(detector.name(), "openrouter");
    }

    #[test]
    fn test_search_queries() {
        let detector = OpenRouterDetector::new();
        let queries = detector.search_queries();
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.contains("OPENROUTER_API_KEY")));
    }
}
