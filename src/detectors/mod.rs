pub mod aws;
pub mod claude;
pub mod gemini;
pub mod github_keys;
pub mod google;
pub mod misc;
pub mod openai;
pub mod shodan;
pub mod slack;
pub mod stripe;
pub mod xai;

pub use aws::AWSDetector;
pub use claude::ClaudeDetector;
pub use gemini::GeminiDetector;
pub use github_keys::GitHubKeysDetector;
pub use google::GoogleDetector;
pub use misc::MiscDetector;
pub use openai::OpenAIDetector;
pub use shodan::ShodanDetector;
pub use slack::SlackDetector;
pub use stripe::StripeDetector;
pub use xai::XAIDetector;

// Re-export for convenience
use crate::core::traits::KeyDetector;

/// Get all available detectors
pub fn all_detectors() -> Vec<Box<dyn KeyDetector>> {
    vec![
        Box::new(ShodanDetector::new()),
        Box::new(OpenAIDetector::new()),
        Box::new(ClaudeDetector::new()),
        Box::new(AWSDetector::new()),
        Box::new(StripeDetector::new()),
        Box::new(SlackDetector::new()),
        Box::new(GoogleDetector::new()),
        Box::new(GeminiDetector::new()),
        Box::new(XAIDetector::new()),
        Box::new(GitHubKeysDetector::new()),
        Box::new(MiscDetector::new()),
    ]
}

/// Get a detector by name
pub fn get_detector(name: &str) -> Option<Box<dyn KeyDetector>> {
    match name.to_lowercase().as_str() {
        "shodan" => Some(Box::new(ShodanDetector::new())),
        "openai" => Some(Box::new(OpenAIDetector::new())),
        "claude" => Some(Box::new(ClaudeDetector::new())),
        "aws" => Some(Box::new(AWSDetector::new())),
        "stripe" => Some(Box::new(StripeDetector::new())),
        "slack" => Some(Box::new(SlackDetector::new())),
        "google" => Some(Box::new(GoogleDetector::new())),
        "gemini" => Some(Box::new(GeminiDetector::new())),
        "xai" => Some(Box::new(XAIDetector::new())),
        "github" | "github_token" => Some(Box::new(GitHubKeysDetector::new())),
        "misc" => Some(Box::new(MiscDetector::new())),
        _ => None,
    }
}
