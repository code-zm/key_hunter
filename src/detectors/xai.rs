use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// xAI API Key pattern: xai- followed by 70-85 alphanumeric characters
    static ref XAI_API: Regex = Regex::new(r"xai-[0-9A-Za-z]{70,85}").unwrap();
}

pub struct XAIDetector {
    patterns: Vec<Regex>,
}

impl XAIDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![XAI_API.clone()],
        }
    }
}

impl Default for XAIDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for XAIDetector {
    fn name(&self) -> &str {
        "xai"
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
                    key_type: "xai".to_string(),
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
            "XAI_API_KEY".to_string(),
            "GROK_API_KEY".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config", ".toml"]
    }
}
