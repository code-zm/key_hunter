use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Google API Key pattern: AIza followed by 35 base64 chars
    static ref GOOGLE_API: Regex = Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap();

    /// Google OAuth pattern
    static ref GOOGLE_OAUTH: Regex = Regex::new(r"[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com").unwrap();

    /// Google OAuth Access Token
    static ref GOOGLE_ACCESS_TOKEN: Regex = Regex::new(r"ya29\.[0-9A-Za-z\-_]+").unwrap();

    /// Google Service Account
    static ref GOOGLE_SERVICE_ACCOUNT: Regex = Regex::new(r#""type":\s*"service_account""#).unwrap();
}

pub struct GoogleDetector {
    patterns: Vec<Regex>,
}

impl GoogleDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                GOOGLE_API.clone(),
                GOOGLE_OAUTH.clone(),
                GOOGLE_ACCESS_TOKEN.clone(),
                GOOGLE_SERVICE_ACCOUNT.clone(),
            ],
        }
    }
}

impl Default for GoogleDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for GoogleDetector {
    fn name(&self) -> &str {
        "google"
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
                    key_type: "google".to_string(),
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
            "AIza".to_string(),
            "GOOGLE_API_KEY".to_string(),
            "GEMINI_API_KEY".to_string(),
            "googleapis.com".to_string(),
            "ya29.".to_string(),
            "service_account".to_string(),
            "AIza extension:env".to_string(),
            "AIza extension:py".to_string(),
            "AIza extension:js".to_string(),
            "googleusercontent.com".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config"]
    }
}
