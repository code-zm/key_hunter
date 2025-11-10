use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Slack Token pattern: xox[pborsa]-... format
    static ref SLACK_TOKEN: Regex = Regex::new(r"(xox[pborsa]-[0-9]{12}-[0-9]{12}-[0-9]{12}-[a-z0-9]{32})").unwrap();

    /// Slack Webhook pattern
    static ref SLACK_WEBHOOK: Regex = Regex::new(r"https://hooks\.slack\.com/services/T[a-zA-Z0-9_]{8}/B[a-zA-Z0-9_]{8}/[a-zA-Z0-9_]{24}").unwrap();
}

pub struct SlackDetector {
    patterns: Vec<Regex>,
}

impl SlackDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![SLACK_TOKEN.clone(), SLACK_WEBHOOK.clone()],
        }
    }
}

impl Default for SlackDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for SlackDetector {
    fn name(&self) -> &str {
        "slack"
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
                    key_type: "slack".to_string(),
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
            "xoxp-".to_string(),
            "xoxb-".to_string(),
            "xoxa-".to_string(),
            "SLACK_TOKEN".to_string(),
            "hooks.slack.com/services".to_string(),
            "slack extension:env".to_string(),
            "xox extension:py".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config"]
    }
}
