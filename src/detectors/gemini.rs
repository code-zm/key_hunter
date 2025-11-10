use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Google/Gemini API Key pattern: AIza followed by 35 base64 chars
    static ref GEMINI_API: Regex = Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap();
}

pub struct GeminiDetector {
    patterns: Vec<Regex>,
}

impl GeminiDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![GEMINI_API.clone()],
        }
    }
}

impl Default for GeminiDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for GeminiDetector {
    fn name(&self) -> &str {
        "gemini"
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
                    key_type: "gemini".to_string(),
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
            "GEMINI_API_KEY".to_string(),
            "generativelanguage.googleapis.com".to_string(),
            "AIza extension:env".to_string(),
            "AIza extension:py".to_string(),
            "AIza extension:js".to_string(),
            "gemini-pro".to_string(),
            "gemini-flash".to_string(),
            "GenerativeModel".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config", ".toml"]
    }
}
