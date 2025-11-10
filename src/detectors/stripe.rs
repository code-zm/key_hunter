use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Stripe API Key pattern
    static ref STRIPE_LIVE: Regex = Regex::new(r"sk_live_[0-9a-zA-Z]{24}").unwrap();

    /// Stripe Restricted API Key pattern
    static ref STRIPE_RESTRICTED: Regex = Regex::new(r"rk_live_[0-9a-zA-Z]{24}").unwrap();
}

pub struct StripeDetector {
    patterns: Vec<Regex>,
}

impl StripeDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![STRIPE_LIVE.clone(), STRIPE_RESTRICTED.clone()],
        }
    }
}

impl Default for StripeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for StripeDetector {
    fn name(&self) -> &str {
        "stripe"
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
                    key_type: "stripe".to_string(),
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
            "sk_live_".to_string(),
            "rk_live_".to_string(),
            "STRIPE_API_KEY".to_string(),
            "stripe extension:env".to_string(),
            "sk_live_ extension:py".to_string(),
            "sk_live_ extension:js".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config"]
    }
}
