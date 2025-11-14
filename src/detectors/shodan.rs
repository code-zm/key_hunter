use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Shodan API keys are 32 alphanumeric characters
    static ref SHODAN_PATTERN: Regex = Regex::new(r"\b[A-Za-z0-9]{32}\b").unwrap();
}

pub struct ShodanDetector {
    patterns: Vec<Regex>,
}

impl ShodanDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![SHODAN_PATTERN.clone()],
        }
    }

    /// Filter out common false positives
    /// Real Shodan keys have:
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one digit
    /// - Good entropy (not a hash)
    fn is_likely_shodan_key(&self, key: &str) -> bool {
        // Must be exactly 32 characters
        if key.len() != 32 {
            return false;
        }

        // Must have uppercase, lowercase, and digits
        let has_upper = PatternUtils::has_mixed_case(key);
        let has_lower = key.chars().any(|c| c.is_lowercase());
        let has_digit = PatternUtils::has_digits(key);

        if !has_upper || !has_lower || !has_digit {
            return false;
        }

        // Filter out MD5 hashes (32 hex chars, but all lowercase)
        if PatternUtils::looks_like_hash(key) {
            return false;
        }

        // Check entropy (Shodan keys have good randomness)
        if !PatternUtils::has_min_entropy(key, 4.0) {
            return false;
        }

        true
    }
}

impl Default for ShodanDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for ShodanDetector {
    fn name(&self) -> &str {
        "shodan"
    }

    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.find_iter(content) {
                let key = capture.as_str();

                if self.is_likely_shodan_key(key) {
                    let (line_number, context) =
                        PatternUtils::get_line_context(content, capture.start(), 2);

                    detected.push(DetectedKey {
                        key: key.to_string(),
                        key_type: "shodan".to_string(),
                        repository: String::new(), // Filled in by search provider
                        file_path: file_path.to_string(),
                        file_url: String::new(), // Filled in by search provider
                        line_number: Some(line_number),
                        context: Some(context),
                    repo_owner_email: None,
                    commit_author_email: None,
                    commit_sha: None,
                    });
                }
            }
        }

        detected
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yml", ".yaml", ".sh", ".go", ".rs"]
    }

    fn search_queries(&self) -> Vec<String> {
        vec![
            "SHODAN_API_KEY".to_string(),
        ]
    }

    fn filter_key(&self, key: &str) -> bool {
        self.is_likely_shodan_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shodan_detector_basic() {
        let detector = ShodanDetector::new();
        assert_eq!(detector.name(), "shodan");
    }

    #[test]
    fn test_detect_valid_key() {
        let detector = ShodanDetector::new();
        // Valid pattern: 32 chars with mixed case and digits
        let content = "SHODAN_API_KEY=oykKBEq2KRySU33OxizNkOir5PgHpMLv";

        let results = detector.detect(content, "test.env");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "oykKBEq2KRySU33OxizNkOir5PgHpMLv");
    }

    #[test]
    fn test_filter_md5_hash() {
        let detector = ShodanDetector::new();
        // MD5 hash (32 hex chars, all lowercase) - should be filtered
        let content = "hash=5d41402abc4b2a76b9719d911017c592";

        let results = detector.detect(content, "test.txt");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_no_uppercase() {
        let detector = ShodanDetector::new();
        // No uppercase - should be filtered
        let content = "key=abcdefghijklmnopqrstuvwxyz123456";

        let results = detector.detect(content, "test.txt");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_queries() {
        let detector = ShodanDetector::new();
        let queries = detector.search_queries();
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.contains("SHODAN_API_KEY")));
    }
}
