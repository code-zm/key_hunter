use crate::core::{DetectedKey, ValidationResult};
use chrono::Utc;
use std::collections::HashMap;

mod template;
pub mod email;

pub use template::TemplateRenderer;
pub use email::{EmailClient, SmtpConfig};

/// Service-specific configuration for issue formatting
#[derive(Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub revoke_url: String,
    pub additional_actions: String,
    pub best_practices: String,
    pub resources: String,
}

impl ServiceConfig {
    pub fn get(key_type: &str) -> Self {
        match key_type {
            "shodan" => Self {
                service_name: "Shodan".to_string(),
                revoke_url: "https://account.shodan.io/".to_string(),
                additional_actions: "".to_string(),
                best_practices: "".to_string(),
                resources: "- [Shodan Account Settings](https://account.shodan.io/)\n".to_string(),
            },
            "claude" => Self {
                service_name: "Anthropic Claude".to_string(),
                revoke_url: "https://console.anthropic.com/settings/keys".to_string(),
                additional_actions: "\n6. **Review API usage logs** for unauthorized access".to_string(),
                best_practices: "".to_string(),
                resources: "- [Anthropic API Keys](https://console.anthropic.com/settings/keys)\n".to_string(),
            },
            "openai" => Self {
                service_name: "OpenAI".to_string(),
                revoke_url: "https://platform.openai.com/api-keys".to_string(),
                additional_actions: "\n6. **Review API usage logs** at https://platform.openai.com/usage".to_string(),
                best_practices: "".to_string(),
                resources: "- [OpenAI API Keys](https://platform.openai.com/api-keys)\n- [OpenAI Usage Dashboard](https://platform.openai.com/usage)\n".to_string(),
            },
            "google" | "gemini" => Self {
                service_name: if key_type == "gemini" { "Google Gemini" } else { "Google Cloud" }.to_string(),
                revoke_url: "https://console.cloud.google.com/apis/credentials".to_string(),
                additional_actions: "\n6. **Review API usage logs** in Google Cloud Console".to_string(),
                best_practices: "\n- Use service accounts with workload identity instead of API keys when possible\n- Implement API key restrictions (referrer restrictions, IP restrictions, API restrictions)".to_string(),
                resources: "- [Google Cloud API Credentials](https://console.cloud.google.com/apis/credentials)\n- [Google Cloud: Best practices for API keys](https://cloud.google.com/docs/authentication/api-keys)\n".to_string(),
            },
            _ => Self {
                service_name: key_type.to_uppercase(),
                revoke_url: format!("your {} account/dashboard", key_type),
                additional_actions: "".to_string(),
                best_practices: "".to_string(),
                resources: "".to_string(),
            },
        }
    }
}

/// Trait for formatting GitHub issues for different key types
pub trait KeyReporter: Send + Sync {
    /// Format the issue title
    fn format_issue_title(&self, detected: &DetectedKey) -> String;

    /// Format the issue body with validation details
    fn format_issue_body(&self, detected: &DetectedKey, validation: &ValidationResult) -> String;

    /// Get the key type this reporter handles
    fn key_type(&self) -> &str;
}

/// Generic issue reporter that works for all key types
pub struct IssueReporter {
    key_type: String,
}

impl IssueReporter {
    pub fn new(key_type: &str) -> Self {
        Self {
            key_type: key_type.to_string(),
        }
    }
}

impl KeyReporter for IssueReporter {
    fn format_issue_title(&self, detected: &DetectedKey) -> String {
        let config = ServiceConfig::get(&self.key_type);
        format!("[Security] Exposed {} API key in {}", config.service_name, detected.file_path)
    }

    fn format_issue_body(&self, detected: &DetectedKey, validation: &ValidationResult) -> String {
        let template = match TemplateRenderer::load("issue") {
            Ok(t) => t,
            Err(_) => return format!("Exposed {} API key found in {} at {}",
                self.key_type.to_uppercase(), detected.file_path, detected.file_url),
        };

        let config = ServiceConfig::get(&self.key_type);
        let mut vars = HashMap::new();

        // Service-specific info
        vars.insert("service_name".to_string(), config.service_name);
        vars.insert("revoke_url".to_string(), config.revoke_url);

        // Basic fields
        vars.insert("file_path".to_string(), detected.file_path.clone());
        vars.insert(
            "line_number".to_string(),
            detected.line_number
                .map(|n| n.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
        );
        vars.insert("file_url".to_string(), detected.file_url.clone());
        vars.insert(
            "key_partial".to_string(),
            if detected.key.len() > 20 {
                format!("{}...", &detected.key[..20])
            } else {
                detected.key.clone()
            },
        );

        // Build metadata section dynamically from validation results
        let mut metadata_lines = Vec::new();
        for (key, value) in &validation.metadata {
            // Format the key nicely (snake_case to Title Case)
            let formatted_key = key
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            metadata_lines.push(format!("- **{}**: {}", formatted_key, value));
        }

        vars.insert("metadata_section".to_string(),
            if metadata_lines.is_empty() {
                "No additional metadata available.".to_string()
            } else {
                metadata_lines.join("\n")
            }
        );

        // Optional sections
        vars.insert("additional_actions".to_string(), config.additional_actions);
        vars.insert("best_practices".to_string(), config.best_practices);
        vars.insert("resources".to_string(), config.resources);

        vars.insert("timestamp".to_string(), Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

        template.render(&vars)
    }

    fn key_type(&self) -> &str {
        &self.key_type
    }
}

/// Get a reporter for a specific key type
pub fn get_reporter(key_type: &str) -> Option<Box<dyn KeyReporter>> {
    Some(Box::new(IssueReporter::new(key_type)))
}

/// Get all available reporters
pub fn all_reporters() -> HashMap<String, Box<dyn KeyReporter>> {
    // Return an empty map since reporters are created on-demand
    HashMap::new()
}
