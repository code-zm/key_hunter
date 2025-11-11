pub mod claude;
pub mod gemini;
pub mod github_keys;
pub mod openai;
pub mod openrouter;
pub mod shodan;
pub mod xai;

pub use claude::ClaudeDetector;
pub use gemini::GeminiDetector;
pub use github_keys::GitHubKeysDetector;
pub use openai::OpenAIDetector;
pub use openrouter::OpenRouterDetector;
pub use shodan::ShodanDetector;
pub use xai::XAIDetector;

// Re-export for convenience
use crate::core::traits::KeyDetector;

/// Get all available detectors
pub fn all_detectors() -> Vec<Box<dyn KeyDetector>> {
    vec![
        Box::new(ShodanDetector::new()),
        Box::new(OpenAIDetector::new()),
        Box::new(OpenRouterDetector::new()),
        Box::new(ClaudeDetector::new()),
        Box::new(GeminiDetector::new()),
        Box::new(XAIDetector::new()),
        Box::new(GitHubKeysDetector::new()),
    ]
}

/// Get a detector by name
pub fn get_detector(name: &str) -> Option<Box<dyn KeyDetector>> {
    match name.to_lowercase().as_str() {
        "shodan" => Some(Box::new(ShodanDetector::new())),
        "openai" => Some(Box::new(OpenAIDetector::new())),
        "openrouter" => Some(Box::new(OpenRouterDetector::new())),
        "claude" => Some(Box::new(ClaudeDetector::new())),
        "gemini" => Some(Box::new(GeminiDetector::new())),
        "xai" => Some(Box::new(XAIDetector::new())),
        "github" | "github_token" => Some(Box::new(GitHubKeysDetector::new())),
        _ => None,
    }
}
