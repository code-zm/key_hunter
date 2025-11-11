pub mod claude;
pub mod gemini;
pub mod github;
pub mod openai;
pub mod openrouter;
pub mod shodan;
pub mod xai;

pub use claude::ClaudeValidator;
pub use gemini::GeminiValidator;
pub use github::GitHubValidator;
pub use openai::OpenAIValidator;
pub use openrouter::OpenRouterValidator;
pub use shodan::ShodanValidator;
pub use xai::XAIValidator;

// Re-export for convenience
use crate::core::traits::KeyValidator;
use std::collections::HashMap;

/// Get all available validators
pub fn all_validators() -> HashMap<String, Box<dyn KeyValidator>> {
    let mut validators: HashMap<String, Box<dyn KeyValidator>> = HashMap::new();
    validators.insert("shodan".to_string(), Box::new(ShodanValidator::new()));
    validators.insert("openai".to_string(), Box::new(OpenAIValidator::new()));
    validators.insert("openrouter".to_string(), Box::new(OpenRouterValidator::new()));
    validators.insert("claude".to_string(), Box::new(ClaudeValidator::new()));
    validators.insert("gemini".to_string(), Box::new(GeminiValidator::new()));
    validators.insert("xai".to_string(), Box::new(XAIValidator::new()));
    validators.insert("github".to_string(), Box::new(GitHubValidator::new()));
    validators
}

/// Get a validator by key type
pub fn get_validator(key_type: &str) -> Option<Box<dyn KeyValidator>> {
    match key_type.to_lowercase().as_str() {
        "shodan" => Some(Box::new(ShodanValidator::new())),
        "openai" => Some(Box::new(OpenAIValidator::new())),
        "openrouter" => Some(Box::new(OpenRouterValidator::new())),
        "claude" => Some(Box::new(ClaudeValidator::new())),
        "gemini" => Some(Box::new(GeminiValidator::new())),
        "xai" => Some(Box::new(XAIValidator::new())),
        "github" | "github_token" => Some(Box::new(GitHubValidator::new())),
        _ => None,
    }
}
