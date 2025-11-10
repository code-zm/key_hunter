use crate::core::results::{HuntResults, ValidatedKey};
use colored::Colorize;

pub struct OutputFormatter;

impl OutputFormatter {
    /// Print a search banner
    pub fn print_banner() {
        println!("{}", "=".repeat(70).bright_cyan());
        println!("{}", "  Key Hunter - Modular API Key Detection Framework".bright_cyan().bold());
        println!("{}", "=".repeat(70).bright_cyan());
        println!();
    }

    /// Print an ethical use warning
    pub fn print_ethical_warning() {
        println!("{}", "⚠️  ETHICAL USE ONLY ⚠️".yellow().bold());
        println!("This tool is for security research and responsible disclosure only.");
        println!("By using this tool, you agree to:");
        println!("  {} Use findings for research and awareness", "✓".green());
        println!("  {} Report all valid keys to owners", "✓".green());
        println!("  {} Not use keys for unauthorized purposes", "✓".green());
        println!();
    }

    /// Print search progress
    pub fn print_search_start(provider: &str, key_type: &str, num_queries: usize) {
        println!("{} Searching {} for {} keys using {} queries...",
            "🔍".bright_yellow(),
            provider.bright_cyan(),
            key_type.bright_green(),
            num_queries.to_string().bright_white()
        );
        println!();
    }

    /// Print a detected key
    pub fn print_detected_key(key: &str, key_type: &str, file_path: &str) {
        println!("  {} Found {} key: {} in {}",
            "✓".green(),
            key_type.bright_yellow(),
            format!("{}...{}", &key[..10], &key[key.len()-4..]).bright_cyan(),
            file_path.bright_white()
        );
    }

    /// Print validation result
    pub fn print_validation_result(validated: &ValidatedKey) {
        if validated.validation.valid {
            println!("    {} VALID! {}",
                "✓".bright_green().bold(),
                Self::format_metadata(&validated.validation.metadata)
            );
        } else {
            println!("    {} Invalid (likely rotated)",
                "✗".bright_black()
            );
        }
    }

    /// Format metadata for display
    fn format_metadata(metadata: &std::collections::HashMap<String, serde_json::Value>) -> String {
        let mut parts = Vec::new();

        if let Some(plan) = metadata.get("plan") {
            parts.push(format!("Plan: {}", plan.as_str().unwrap_or("unknown").bright_cyan()));
        }

        if let Some(credits) = metadata.get("query_credits") {
            parts.push(format!("Credits: {}", credits.to_string().bright_yellow()));
        }

        parts.join(", ")
    }

    /// Print final statistics
    pub fn print_statistics(results: &HuntResults) {
        println!();
        println!("{}", "=".repeat(70).bright_cyan());
        println!("{}", "  Results Summary".bright_cyan().bold());
        println!("{}", "=".repeat(70).bright_cyan());
        println!();

        println!("  Total keys found: {}", results.total_keys_found.to_string().bright_white());
        println!("  Valid keys: {}", results.valid_keys.len().to_string().bright_green());
        println!("  Invalid keys: {}", results.invalid_keys.len().to_string().bright_red());
        println!();

        if !results.by_key_type.is_empty() {
            println!("  {} Keys by type:", "📊".bright_yellow());
            for (key_type, count) in &results.by_key_type {
                println!("    {}: {}", key_type.bright_cyan(), count.to_string().bright_white());
            }
            println!();
        }

        println!("  {} Statistics:", "📈".bright_yellow());
        println!("    Files attempted: {}", results.statistics.files_attempted.to_string().bright_white());
        println!("    Files from snippets: {}", results.statistics.files_from_snippets.to_string().bright_green());
        println!("    Files downloaded: {}", results.statistics.files_downloaded.to_string().bright_yellow());
        println!("    Files not found (404): {}", results.statistics.files_404.to_string().bright_red());
        println!();

        if !results.valid_keys.is_empty() {
            println!("{}", "⚠️  VALID KEYS FOUND - RESPONSIBLE DISCLOSURE REQUIRED".yellow().bold());
            println!("Next steps:");
            println!("  1. Report to repository owners");
            println!("  2. Report to service providers");
            println!("  3. Document findings");
            println!("  4. DO NOT use keys for unauthorized purposes");
        }

        println!();
        println!("{}", "=".repeat(70).bright_cyan());
    }

    /// Print error message
    pub fn print_error(message: &str) {
        eprintln!("{} {}", "❌".bright_red(), message.red());
    }

    /// Print warning message
    pub fn print_warning(message: &str) {
        println!("{} {}", "⚠️".bright_yellow(), message.yellow());
    }

    /// Print success message
    pub fn print_success(message: &str) {
        println!("{} {}", "✓".bright_green(), message.green());
    }

    /// Print info message
    pub fn print_info(message: &str) {
        println!("{} {}", "ℹ️".bright_blue(), message);
    }
}
