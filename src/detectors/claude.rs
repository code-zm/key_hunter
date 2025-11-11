use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Claude API Key pattern: sk-ant-api03- followed by base64-like chars
    static ref CLAUDE_API: Regex = Regex::new(r"sk-ant-api03-[A-Za-z0-9_-]{95,110}").unwrap();
}

pub struct ClaudeDetector {
    patterns: Vec<Regex>,
}

impl ClaudeDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![CLAUDE_API.clone()],
        }
    }
}

impl Default for ClaudeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for ClaudeDetector {
    fn name(&self) -> &str {
        "claude"
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
                    key_type: "claude".to_string(),
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
            "ANTHROPIC_API_KEY".to_string(),
            "CLAUDE_API_KEY".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config"]
    }
}
