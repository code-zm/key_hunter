use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// AWS Access Key ID pattern: AKIA followed by 16 alphanumeric characters
    static ref AWS_ACCESS_KEY: Regex = Regex::new(r"((?:A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16})").unwrap();

    /// AWS AppSync GraphQL Key pattern
    static ref AWS_APPSYNC: Regex = Regex::new(r"da2-[a-z0-9]{26}").unwrap();
}

pub struct AWSDetector {
    patterns: Vec<Regex>,
}

impl AWSDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![AWS_ACCESS_KEY.clone(), AWS_APPSYNC.clone()],
        }
    }
}

impl Default for AWSDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for AWSDetector {
    fn name(&self) -> &str {
        "aws"
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
                    key_type: "aws".to_string(),
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

    fn search_queries(&self) -> Vec<String> {
        vec![
            "AKIA".to_string(),
            "AWS_ACCESS_KEY_ID".to_string(),
            "aws_access_key_id extension:env".to_string(),
            "AKIA extension:py".to_string(),
            "AKIA extension:js".to_string(),
            "AKIA extension:json".to_string(),
            "AWS_SECRET_ACCESS_KEY".to_string(),
            "da2- AND appsync".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config", ".ini"]
    }
}
