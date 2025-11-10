use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use key_hunter::cli::{Cli, Commands, OutputFormatter};
use key_hunter::core::{Config, DetectedKey, HuntResults, SearchQuery, ValidatedKey};
use key_hunter::detectors;
use key_hunter::providers::GitHubProvider;
use key_hunter::validators;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    let _ = dotenv::dotenv();

    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    // Print banner
    OutputFormatter::print_banner();

    // Execute command
    if let Err(e) = execute_command(cli.command).await {
        OutputFormatter::print_error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}

async fn execute_command(command: Commands) -> key_hunter::Result<()> {
    match command {
        Commands::Search {
            provider,
            key_type,
            query,
            max_results,
            num_queries,
            github_token,
            output,
            test_immediately,
            auto_split,
        } => {
            search_command(
                provider,
                key_type,
                query,
                max_results,
                num_queries,
                github_token,
                output,
                test_immediately,
                auto_split,
            )
            .await?;
        }
        Commands::Validate {
            input,
            output,
            key_type,
        } => {
            validate_command(input, output, key_type).await?;
        }
        Commands::Test { key, key_type } => {
            test_command(key, key_type).await?;
        }
        Commands::Report {
            input,
            format,
            output,
        } => {
            report_command(input, format, output)?;
        }
        Commands::List { what } => {
            list_command(what)?;
        }
    }

    Ok(())
}

fn load_config() -> key_hunter::Result<Config> {
    // Try to load from config/default.toml first
    let config_paths = vec!["config/default.toml", "default.toml", ".key_hunter.toml"];

    for path in config_paths {
        if Path::new(path).exists() {
            match fs::read_to_string(path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            info!("Loaded config from {}", path);
                            return Ok(config);
                        }
                        Err(e) => {
                            warn!("Failed to parse config from {}: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read config from {}: {}", path, e);
                }
            }
        }
    }

    // Return default config if no file found
    warn!("No config file found, using defaults");
    Ok(Config::default())
}

/// Generate search qualifiers for different file types
/// GitHub Code Search doesn't support date filtering, so we split by extension/language instead
fn generate_extension_qualifiers() -> Vec<String> {
    vec![
        // Common configuration files
        "extension:env".to_string(),
        "extension:txt".to_string(),
        "extension:cfg".to_string(),
        "extension:conf".to_string(),
        "extension:config".to_string(),
        "extension:ini".to_string(),
        "extension:toml".to_string(),
        "extension:yaml".to_string(),
        "extension:yml".to_string(),
        "extension:json".to_string(),
        "extension:xml".to_string(),

        // Environment/config file variations (no extension)
        "filename:.env".to_string(),
        "filename:env.txt".to_string(),
        "filename:.env.local".to_string(),
        "filename:.env.development".to_string(),
        "filename:.env.production".to_string(),
        "filename:config".to_string(),

        // Programming language files
        "extension:py".to_string(),      // Python
        "extension:js".to_string(),      // JavaScript
        "extension:ts".to_string(),      // TypeScript
        "extension:jsx".to_string(),     // React
        "extension:tsx".to_string(),     // TypeScript React
        "extension:rb".to_string(),      // Ruby
        "extension:go".to_string(),      // Go
        "extension:java".to_string(),    // Java
        "extension:kt".to_string(),      // Kotlin
        "extension:swift".to_string(),   // Swift
        "extension:rs".to_string(),      // Rust
        "extension:php".to_string(),     // PHP
        "extension:cs".to_string(),      // C#
        "extension:cpp".to_string(),     // C++
        "extension:c".to_string(),       // C
        "extension:h".to_string(),       // C/C++ headers
        "extension:m".to_string(),       // Objective-C
        "extension:sh".to_string(),      // Shell scripts
        "extension:bash".to_string(),    // Bash scripts
        "extension:zsh".to_string(),     // Zsh scripts
        "extension:pl".to_string(),      // Perl
        "extension:r".to_string(),       // R
        "extension:scala".to_string(),   // Scala
        "extension:clj".to_string(),     // Clojure
        "extension:ex".to_string(),      // Elixir
        "extension:exs".to_string(),     // Elixir scripts
        "extension:erl".to_string(),     // Erlang
        "extension:dart".to_string(),    // Dart
        "extension:lua".to_string(),     // Lua
        "extension:vim".to_string(),     // Vim script

        // Web/markup files
        "extension:html".to_string(),
        "extension:htm".to_string(),
        "extension:vue".to_string(),     // Vue
        "extension:svelte".to_string(),  // Svelte

        // Documentation files
        "extension:md".to_string(),      // Markdown
        "extension:rst".to_string(),     // reStructuredText
        "extension:adoc".to_string(),    // AsciiDoc

        // Infrastructure/DevOps files
        "extension:dockerfile".to_string(),
        "filename:Dockerfile".to_string(),
        "filename:docker-compose.yml".to_string(),
        "filename:docker-compose.yaml".to_string(),
        "extension:tf".to_string(),      // Terraform
        "extension:tfvars".to_string(),  // Terraform variables
        "extension:hcl".to_string(),     // HashiCorp Config

        // CI/CD files
        "filename:.gitlab-ci.yml".to_string(),
        "filename:.travis.yml".to_string(),
        "filename:circle.yml".to_string(),
        "filename:azure-pipelines.yml".to_string(),
        "path:.github/workflows".to_string(),

        // Package/build files
        "filename:package.json".to_string(),
        "filename:composer.json".to_string(),
        "filename:Gemfile".to_string(),
        "filename:Cargo.toml".to_string(),
        "filename:go.mod".to_string(),
        "filename:pom.xml".to_string(),
        "filename:build.gradle".to_string(),
        "filename:requirements.txt".to_string(),

        // Notebook files
        "extension:ipynb".to_string(),   // Jupyter notebooks

        // Other common files
        "extension:log".to_string(),
        "extension:properties".to_string(),
    ]
}

async fn search_command(
    provider: String,
    key_type: String,
    custom_query: Option<String>,
    _max_results: usize,
    num_queries: usize,
    github_token: Option<String>,
    output_file: Option<String>,
    test_immediately: bool,
    auto_split: bool,
) -> key_hunter::Result<()> {
    OutputFormatter::print_ethical_warning();

    // Load config
    let config = load_config()?;

    // Get GitHub token from CLI arg or environment variable
    let token = github_token.or_else(|| std::env::var("GITHUB_TOKEN").ok());

    // Get the appropriate search provider
    let search_provider: Box<dyn key_hunter::SearchProvider> = match provider.as_str() {
        "github" => {
            if let Some(github_config) = config.github {
                Box::new(GitHubProvider::with_config(
                    token,
                    github_config.base_url,
                    github_config.rate_limit_delay_ms,
                ))
            } else {
                Box::new(GitHubProvider::new(token))
            }
        }
        _ => {
            return Err(key_hunter::KeyHunterError::Config(format!(
                "Unknown provider: {}",
                provider
            )));
        }
    };

    // Get detectors
    let detectors: Vec<Box<dyn key_hunter::KeyDetector>> = if key_type == "all" {
        detectors::all_detectors()
    } else {
        vec![detectors::get_detector(&key_type).ok_or_else(|| {
            key_hunter::KeyHunterError::Config(format!("Unknown key type: {}", key_type))
        })?]
    };

    // Get validators if testing immediately
    let validators = if test_immediately {
        Some(validators::all_validators())
    } else {
        None
    };

    OutputFormatter::print_search_start(&provider, &key_type, num_queries);

    let mut all_results = HuntResults::default();

    // Search for each detector
    for detector in &detectors {
        info!("Searching for {} keys", detector.name());

        // Get search queries
        let queries: Vec<String> = if let Some(ref q) = custom_query {
            vec![q.clone()]
        } else {
            detector
                .search_queries()
                .into_iter()
                .take(num_queries)
                .collect()
        };

        // Execute each query
        for (idx, query_str) in queries.iter().enumerate() {
            println!(
                "\n{} Query {}/{}: {}",
                "🔍".bright_yellow(),
                idx + 1,
                queries.len(),
                query_str.bright_cyan()
            );

            // Determine if we should auto-split this query
            let queries_to_run: Vec<String> = if auto_split {
                // Generate extension/file qualifiers
                let qualifiers = generate_extension_qualifiers();
                info!("Auto-split enabled: will search across {} file type variations", qualifiers.len());

                qualifiers
                    .iter()
                    .map(|qualifier| {
                        format!("{} {}", query_str, qualifier)
                    })
                    .collect()
            } else {
                vec![query_str.clone()]
            };

            let mut total_results = Vec::new();

            // Execute the query (or multiple queries if auto-split)
            for (sub_idx, sub_query_str) in queries_to_run.iter().enumerate() {
                if auto_split && sub_idx > 0 {
                    println!("  {} File type {}/{}", "📂".bright_blue(), sub_idx + 1, queries_to_run.len());
                }

                let query = SearchQuery {
                    query: sub_query_str.clone(),
                    max_results: 1000, // Always fetch max when auto-splitting
                    file_extensions: detector
                        .file_extensions()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                };

                // Search
                match search_provider.search(&query).await {
                    Ok(search_results) => {
                        let count = search_results.len();
                        total_results.extend(search_results);

                        if auto_split {
                            println!("    {} Found {} files (total: {})",
                                "✓".green(), count, total_results.len());
                        }
                    }
                    Err(e) => {
                        warn!("Failed to search: {}", e);
                        if !auto_split {
                            return Err(e);
                        }
                        // Continue with next month if auto-splitting
                        continue;
                    }
                }

                // Small delay between sub-queries
                if auto_split && sub_idx < queries_to_run.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }

            // Deduplicate results by file URL (same file can appear in multiple queries when auto-splitting)
            let total_before_dedup = total_results.len();
            let mut seen_urls = HashSet::new();
            let search_results: Vec<_> = total_results
                .into_iter()
                .filter(|result| seen_urls.insert(result.file_url.clone()))
                .collect();

            if auto_split && total_before_dedup != search_results.len() {
                let duplicates_removed = total_before_dedup - search_results.len();
                println!("  {} Removed {} duplicate files", "✓".green(), duplicates_removed);
            }

            println!("  {} Found {} files total", "✓".green(), search_results.len());

            // Progress bar
            let pb = ProgressBar::new(search_results.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("=>-"),
            );

            // Process each file
            for search_result in search_results {
                all_results.statistics.files_attempted += 1;

                // Use text matches if available (much faster - no download needed!)
                let content = if let Some(ref snippets) = search_result.text_matches {
                    // Join all snippets - the key should be in here
                    all_results.statistics.files_from_snippets += 1;
                    snippets.join("\n")
                } else {
                    // Fallback: download file if no snippets (shouldn't happen with text-match API)
                    match search_provider.get_file_content(&search_result).await {
                        Ok(c) => {
                            all_results.statistics.files_downloaded += 1;
                            c
                        }
                        Err(e) => {
                            if e.to_string().contains("404") || e.to_string().contains("not found") {
                                all_results.statistics.files_404 += 1;
                            } else {
                                all_results.statistics.files_other_error += 1;
                            }
                            pb.inc(1);
                            continue;
                        }
                    }
                };

                // Detect keys from snippet content
                let mut detected_keys = detector.detect(&content, &search_result.file_path);

                // Fill in repository and URL info
                for key in &mut detected_keys {
                    key.repository = search_result.repository.clone();
                    key.file_url = search_result.file_url.clone();
                }

                // Process detected keys
                for detected_key in detected_keys {
                    all_results.statistics.keys_found += 1;
                    OutputFormatter::print_detected_key(
                        &detected_key.key,
                        &detected_key.key_type,
                        &detected_key.file_path,
                    );

                    // Validate immediately if requested
                    if let Some(ref validators) = validators {
                        if let Some(validator) = validators.get(&detected_key.key_type) {
                            all_results.statistics.keys_tested += 1;

                            // Rate limit
                            tokio::time::sleep(validator.rate_limit()).await;

                            match validator.validate(&detected_key.key).await {
                                Ok(validation) => {
                                    let validated = ValidatedKey {
                                        detected: detected_key.clone(),
                                        validation: validation.clone(),
                                        validated_at: Utc::now(),
                                    };

                                    OutputFormatter::print_validation_result(&validated);

                                    if validation.valid {
                                        all_results.statistics.keys_valid += 1;
                                        all_results.valid_keys.push(validated);
                                        *all_results
                                            .by_key_type
                                            .entry(detected_key.key_type.clone())
                                            .or_insert(0) += 1;
                                    } else {
                                        all_results.statistics.keys_invalid += 1;
                                        all_results.invalid_keys.push(validated);
                                    }
                                }
                                Err(e) => {
                                    OutputFormatter::print_warning(&format!(
                                        "Validation error: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }
                }

                pb.inc(1);
            }

            pb.finish_and_clear();

            // Rate limit between queries
            if idx < queries.len() - 1 {
                OutputFormatter::print_info("Waiting 5 seconds before next query...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    all_results.timestamp = Utc::now();
    all_results.total_keys_found = all_results.statistics.keys_found;

    // Generate output filename with timestamp and directory structure
    let output_path = if let Some(path) = output_file {
        Path::new(&path).to_path_buf()
    } else {
        // Default: results/<key-type>/valid_keys_<timestamp>.json
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let dir = Path::new("results").join(&key_type);
        fs::create_dir_all(&dir)?;
        dir.join(format!("valid_keys_{}.json", timestamp))
    };

    // Create results structure with only valid keys
    let valid_only_results = serde_json::json!({
        "timestamp": all_results.timestamp,
        "key_type": key_type,
        "total_valid_keys": all_results.valid_keys.len(),
        "total_keys_scanned": all_results.total_keys_found,
        "valid_keys": all_results.valid_keys,
    });

    // Save results (only valid keys)
    let json = serde_json::to_string_pretty(&valid_only_results)?;
    fs::write(&output_path, json)?;

    OutputFormatter::print_statistics(&all_results);
    OutputFormatter::print_success(&format!("Results saved to {}", output_path.display()));

    Ok(())
}

async fn validate_command(
    input: String,
    output: String,
    key_type: String,
) -> key_hunter::Result<()> {
    OutputFormatter::print_info(&format!("Loading keys from {}", input));

    // Load detected keys from file
    let json = fs::read_to_string(&input)?;
    let detected_keys: Vec<DetectedKey> = serde_json::from_str(&json)?;

    println!("Loaded {} keys to validate", detected_keys.len());

    // Get validators
    let validators = validators::all_validators();

    let mut results = HuntResults::default();

    // Progress bar
    let pb = ProgressBar::new(detected_keys.len() as u64);

    for detected_key in detected_keys {
        // Skip if not matching key type filter
        if key_type != "all" && detected_key.key_type != key_type {
            pb.inc(1);
            continue;
        }

        if let Some(validator) = validators.get(&detected_key.key_type) {
            // Rate limit
            tokio::time::sleep(validator.rate_limit()).await;

            match validator.validate(&detected_key.key).await {
                Ok(validation) => {
                    let validated = ValidatedKey {
                        detected: detected_key.clone(),
                        validation: validation.clone(),
                        validated_at: Utc::now(),
                    };

                    if validation.valid {
                        results.valid_keys.push(validated);
                        *results
                            .by_key_type
                            .entry(detected_key.key_type.clone())
                            .or_insert(0) += 1;
                    } else {
                        results.invalid_keys.push(validated);
                    }
                }
                Err(e) => {
                    error!("Validation error for {}: {}", detected_key.key, e);
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    results.timestamp = Utc::now();
    results.total_keys_found = results.valid_keys.len() + results.invalid_keys.len();

    // Save results
    let json = serde_json::to_string_pretty(&results)?;
    fs::write(&output, json)?;

    OutputFormatter::print_statistics(&results);
    OutputFormatter::print_success(&format!("Results saved to {}", output));

    Ok(())
}

async fn test_command(key: String, key_type: String) -> key_hunter::Result<()> {
    OutputFormatter::print_info(&format!("Testing {} key...", key_type));

    let validator = validators::get_validator(&key_type).ok_or_else(|| {
        key_hunter::KeyHunterError::Config(format!("Unknown key type: {}", key_type))
    })?;

    match validator.validate(&key).await {
        Ok(validation) => {
            if validation.valid {
                OutputFormatter::print_success("Key is VALID!");
                println!("\nMetadata:");
                for (k, v) in &validation.metadata {
                    println!("  {}: {}", k.bright_cyan(), v.to_string().bright_white());
                }
            } else {
                OutputFormatter::print_error(&format!(
                    "Key is invalid: {}",
                    validation.error.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
        }
        Err(e) => {
            OutputFormatter::print_error(&format!("Validation failed: {}", e));
        }
    }

    Ok(())
}

fn report_command(input: String, format: String, output: Option<String>) -> key_hunter::Result<()> {
    OutputFormatter::print_info(&format!("Generating {} report from {}", format, input));

    let json = fs::read_to_string(&input)?;
    let results: HuntResults = serde_json::from_str(&json)?;

    let report = match format.as_str() {
        "json" => serde_json::to_string_pretty(&results)?,
        "text" => generate_text_report(&results),
        _ => {
            return Err(key_hunter::KeyHunterError::Config(format!(
                "Unknown format: {}",
                format
            )));
        }
    };

    if let Some(output_file) = output {
        fs::write(&output_file, report)?;
        OutputFormatter::print_success(&format!("Report saved to {}", output_file));
    } else {
        println!("\n{}", report);
    }

    Ok(())
}

fn list_command(what: String) -> key_hunter::Result<()> {
    match what.as_str() {
        "detectors" | "all" => {
            println!("{}", "Available Detectors:".bright_cyan().bold());
            let detectors = detectors::all_detectors();
            for detector in detectors {
                println!("  {} {}", "•".bright_yellow(), detector.name().bright_white());
            }
            println!();
        }
        _ => {}
    }

    match what.as_str() {
        "validators" | "all" => {
            println!("{}", "Available Validators:".bright_cyan().bold());
            let validators = validators::all_validators();
            for (name, _) in validators {
                println!("  {} {}", "•".bright_yellow(), name.bright_white());
            }
            println!();
        }
        _ => {}
    }

    Ok(())
}

fn generate_text_report(results: &HuntResults) -> String {
    let mut report = String::new();

    report.push_str(&format!("Key Hunter Report\n"));
    report.push_str(&format!("Generated: {}\n\n", results.timestamp));

    report.push_str(&format!("Summary:\n"));
    report.push_str(&format!("  Total keys: {}\n", results.total_keys_found));
    report.push_str(&format!("  Valid: {}\n", results.valid_keys.len()));
    report.push_str(&format!("  Invalid: {}\n\n", results.invalid_keys.len()));

    if !results.valid_keys.is_empty() {
        report.push_str("Valid Keys:\n");
        for (idx, key) in results.valid_keys.iter().enumerate() {
            report.push_str(&format!("\n[{}] {}\n", idx + 1, key.detected.key_type));
            report.push_str(&format!("  Repository: {}\n", key.detected.repository));
            report.push_str(&format!("  File: {}\n", key.detected.file_path));
            report.push_str(&format!("  URL: {}\n", key.detected.file_url));
        }
    }

    report
}
