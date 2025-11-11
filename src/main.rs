use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use key_hunter::cli::{Cli, Commands, OutputFormatter};
use key_hunter::core::{Config, DetectedKey, HuntResults, SearchQuery, ValidatedKey};
use key_hunter::detectors;
use key_hunter::providers::GitHubProvider;
use key_hunter::validators;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    let _ = dotenv::dotenv();

    let cli = Cli::parse();

    // Initialize logging
    // In non-verbose mode, only show errors so progress bars work cleanly
    let log_level = if cli.verbose { "debug" } else { "error" };
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
            output,
            validate,
        } => {
            search_command(
                provider,
                key_type,
                query,
                output,
                validate,
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
        Commands::List { what } => {
            list_command(what)?;
        }
        Commands::Report {
            results_dir,
            key_type,
            dry_run,
        } => {
            report_command(results_dir, key_type, dry_run).await?;
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
    output_file: Option<String>,
    validate: bool,
) -> key_hunter::Result<()> {
    OutputFormatter::print_ethical_warning();

    // Load config
    let config = load_config()?;

    // Get GitHub tokens from environment - supports GITHUB_TOKEN1 through GITHUB_TOKEN5
    let mut tokens = Vec::new();
    for i in 1..=5 {
        if let Ok(token) = std::env::var(format!("GITHUB_TOKEN{}", i)) {
            if !token.is_empty() {
                tokens.push(token);
            }
        }
    }

    if !tokens.is_empty() {
        info!("Using {} GitHub token(s)", tokens.len());
    } else {
        warn!("No GitHub tokens found (GITHUB_TOKEN1-5). Running unauthenticated with severe rate limits.");
    }

    // Get the appropriate search provider
    let search_provider: Box<dyn key_hunter::SearchProvider> = match provider.as_str() {
        "github" => {
            let github_config = config.github.unwrap_or_default();
            Box::new(GitHubProvider::with_config(
                tokens,
                github_config.base_url,
                github_config.rate_limit_delay_ms,
            ))
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

    // Get validators if validating immediately
    let validators = if validate {
        let validators_config = config.validators.unwrap_or_default();
        Some(validators::all_validators(&validators_config))
    } else {
        None
    };

    let mut all_results = HuntResults::default();

    // Search for each detector
    for detector in &detectors {
        info!("Searching for {} keys", detector.name());

        // Get search queries - use all available queries
        let queries: Vec<String> = if let Some(ref q) = custom_query {
            vec![q.clone()]
        } else {
            detector.search_queries()
        };

        OutputFormatter::print_search_start(&provider, &key_type, queries.len());

        // Execute each query
        for (idx, query_str) in queries.iter().enumerate() {
            println!(
                "\n{} Query {}/{}: {}",
                "🔍".bright_yellow(),
                idx + 1,
                queries.len(),
                query_str.bright_cyan()
            );

            // Auto-split queries by file type to bypass GitHub's 1000 result limit
            let qualifiers = generate_extension_qualifiers();

            let queries_to_run: Vec<String> = qualifiers
                .iter()
                .map(|qualifier| {
                    format!("{} {}", query_str, qualifier)
                })
                .collect();

            // Create multi-progress for spinner + progress bar
            let multi = MultiProgress::new();

            // Spinner for status messages
            let spinner = multi.add(ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}\n")
                    .unwrap()
            );
            spinner.enable_steady_tick(Duration::from_millis(100));

            // Progress bar for file type search
            let search_pb = multi.add(ProgressBar::new(queries_to_run.len() as u64));
            search_pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("=>-"),
            );

            let mut total_results = Vec::new();

            // Execute the split queries
            for (sub_idx, sub_query_str) in queries_to_run.iter().enumerate() {
                let qualifier = &qualifiers[sub_idx];
                spinner.set_message(format!("Searching {} | Total: {}",
                    qualifier.green(),
                    total_results.len().to_string().green()
                ));

                let query = SearchQuery {
                    query: sub_query_str.clone(),
                    max_results: 1000, // GitHub's max
                    file_extensions: detector
                        .file_extensions()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                };

                // Search
                match search_provider.search(&query).await {
                    Ok(search_results) => {
                        total_results.extend(search_results);
                        spinner.set_message(format!("Searching {} | Total: {}",
                            qualifier.green(),
                            total_results.len().to_string().green()
                        ));
                    }
                    Err(e) => {
                        spinner.set_message(format!("Error: {} | Total: {}",
                            e,
                            total_results.len().to_string().green()
                        ));
                        // Continue with next file type
                    }
                }

                search_pb.inc(1);

                // Small delay between sub-queries
                if sub_idx < queries_to_run.len() - 1 {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }

            spinner.finish_and_clear();
            search_pb.finish_and_clear();

            // Deduplicate results by file URL (same file can appear in multiple queries)
            let total_before_dedup = total_results.len();
            let mut seen_urls = HashSet::new();
            let search_results: Vec<_> = total_results
                .into_iter()
                .filter(|result| seen_urls.insert(result.file_url.clone()))
                .collect();

            let duplicates_removed = total_before_dedup - search_results.len();
            println!("  {} Found {} unique files ({} duplicates removed)",
                "✓".green(), search_results.len(), duplicates_removed);

            // Create multi-progress for file scanning
            let scan_multi = MultiProgress::new();

            // Spinner for validation status if validating
            let scan_spinner = if validators.is_some() {
                let spinner = scan_multi.add(ProgressBar::new_spinner());
                spinner.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} {msg}\n")
                        .unwrap()
                );
                spinner.enable_steady_tick(Duration::from_millis(100));
                Some(spinner)
            } else {
                None
            };

            // Progress bar for file scanning
            let pb = scan_multi.add(ProgressBar::new(search_results.len() as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("=>-"),
            );

            // Process each file
            for search_result in search_results {
                all_results.statistics.files_attempted += 1;

                // Update spinner: searching for keys
                if let Some(ref spinner) = scan_spinner {
                    spinner.set_message(format!("Searching for keys | Valid: {}",
                        all_results.statistics.keys_valid.to_string().green()
                    ));
                }

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

                    // Update spinner: key found
                    if let Some(ref spinner) = scan_spinner {
                        spinner.set_message(format!("Key found | Valid: {}",
                            all_results.statistics.keys_valid.to_string().green()
                        ));
                    }

                    // Validate immediately if requested
                    if let Some(ref validators) = validators {
                        if let Some(validator) = validators.get(&detected_key.key_type) {
                            all_results.statistics.keys_tested += 1;

                            // Truncate key for display
                            let key_preview = if detected_key.key.len() > 20 {
                                format!("{}...", &detected_key.key[..20])
                            } else {
                                detected_key.key.clone()
                            };

                            // Update spinner: validating key
                            if let Some(ref spinner) = scan_spinner {
                                spinner.set_message(format!("Validating {} | Valid: {}",
                                    key_preview.green(),
                                    all_results.statistics.keys_valid.to_string().green()
                                ));
                            }

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
                                Err(_e) => {
                                    // Silently continue - spinner shows overall progress
                                }
                            }
                        }
                    }
                }

                pb.inc(1);
            }

            if let Some(spinner) = scan_spinner {
                spinner.finish_and_clear();
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

    // Load config
    let config = load_config()?;

    // Load detected keys from file
    let json = fs::read_to_string(&input)?;
    let detected_keys: Vec<DetectedKey> = serde_json::from_str(&json)?;

    println!("Loaded {} keys to validate", detected_keys.len());

    // Get validators
    let validators_config = config.validators.unwrap_or_default();
    let validators = validators::all_validators(&validators_config);

    let mut results = HuntResults::default();

    // Create multi-progress for validation
    let val_multi = MultiProgress::new();

    // Spinner for validation status
    let val_spinner = val_multi.add(ProgressBar::new_spinner());
    val_spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\n")
            .unwrap()
    );
    val_spinner.enable_steady_tick(Duration::from_millis(100));

    // Progress bar for validation
    let pb = val_multi.add(ProgressBar::new(detected_keys.len() as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    for detected_key in detected_keys {
        // Skip if not matching key type filter
        if key_type != "all" && detected_key.key_type != key_type {
            pb.inc(1);
            continue;
        }

        if let Some(validator) = validators.get(&detected_key.key_type) {
            // Truncate key for display
            let key_preview = if detected_key.key.len() > 20 {
                format!("{}...", &detected_key.key[..20])
            } else {
                detected_key.key.clone()
            };

            // Update spinner: validating key
            val_spinner.set_message(format!("Validating {} | Valid: {}",
                key_preview.green(),
                results.valid_keys.len().to_string().green()
            ));

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
                Err(_e) => {
                    // Silently continue - spinner shows overall progress
                }
            }
        }

        pb.inc(1);
    }

    val_spinner.finish_and_clear();
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

    // Load config
    let config = load_config()?;
    let validators_config = config.validators.unwrap_or_default();

    let validator = validators::get_validator(&key_type, &validators_config).ok_or_else(|| {
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
            let config = load_config()?;
            let validators_config = config.validators.unwrap_or_default();
            let validators = validators::all_validators(&validators_config);

            // Sort validators to match detector order
            let detector_order: Vec<String> = detectors::all_detectors()
                .iter()
                .map(|d| d.name().to_string())
                .collect();

            let mut sorted_validators: Vec<String> = validators.keys()
                .cloned()
                .collect();

            sorted_validators.sort_by_key(|name| {
                detector_order.iter()
                    .position(|d| d == name)
                    .unwrap_or(usize::MAX)
            });

            for name in sorted_validators {
                println!("  {} {}", "•".bright_yellow(), name.bright_white());
            }
            println!();
        }
        _ => {}
    }

    Ok(())
}

async fn report_command(
    results_dir: String,
    key_type_filter: String,
    dry_run: bool,
) -> key_hunter::Result<()> {
    // Load GitHub token from environment
    let github_token = std::env::var("ISSUES_GITHUB_TOKEN").map_err(|_| {
        key_hunter::KeyHunterError::Config(
            "ISSUES_GITHUB_TOKEN environment variable not set. Please set it in your .env file.".to_string()
        )
    })?;

    if dry_run {
        OutputFormatter::print_info("Running in DRY RUN mode - no issues will be created\n");
    }

    OutputFormatter::print_info(&format!("Reading results from {}\n", results_dir));

    // Read all JSON files from results directory
    let results_path = Path::new(&results_dir);
    if !results_path.exists() {
        return Err(key_hunter::KeyHunterError::NotFound(format!(
            "Results directory not found: {}",
            results_dir
        )));
    }

    let mut all_validated_keys = Vec::new();

    // Iterate through each key type directory
    for entry in fs::read_dir(results_path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Read all JSON files in this key type directory
        for json_entry in fs::read_dir(&path)? {
            let json_entry = json_entry?;
            let json_path = json_entry.path();

            if json_path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Read and parse JSON file
            match fs::read_to_string(&json_path) {
                Ok(json_content) => {
                    #[derive(serde::Deserialize)]
                    struct ResultFile {
                        key_type: String,
                        valid_keys: Vec<ValidatedKey>,
                    }

                    match serde_json::from_str::<ResultFile>(&json_content) {
                        Ok(result_file) => {
                            // Filter by key type if specified
                            if key_type_filter == "all" || result_file.key_type == key_type_filter {
                                let count = result_file.valid_keys.len();
                                all_validated_keys.extend(result_file.valid_keys);
                                println!(
                                    "Loaded {} keys from {}",
                                    count,
                                    json_path.display()
                                );
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse {}: {}", json_path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read {}: {}", json_path.display(), e);
                }
            }
        }
    }

    if all_validated_keys.is_empty() {
        OutputFormatter::print_info("\nNo valid keys found to report");
        return Ok(());
    }

    println!(
        "\nFound {} validated keys across all result files\n",
        all_validated_keys.len()
    );

    // Create GitHub issue client
    let issue_client = key_hunter::GitHubIssueClient::new(github_token, dry_run);

    // Set up progress bars
    let report_multi = MultiProgress::new();

    // Spinner for status messages
    let spinner = report_multi.add(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\n")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Creating GitHub issues...");

    // Progress bar for issue creation
    let progress_bar = report_multi.add(ProgressBar::new(all_validated_keys.len() as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Create issues
    let stats = issue_client
        .create_issues_bulk(&all_validated_keys, Some(&progress_bar))
        .await?;

    // Cleanup progress bars
    spinner.finish_and_clear();
    progress_bar.finish_and_clear();

    // Print summary
    println!("\n{}", "=".repeat(80));
    println!("Summary:");
    println!("   Total keys: {}", stats.total);
    println!("   Issues created: {}", stats.success);
    println!("   Failed: {}", stats.failed);
    println!("   Skipped: {}", stats.skipped);

    if !stats.errors.is_empty() {
        println!("\nErrors:");
        for error in &stats.errors {
            println!("   {}", error);
        }
    }

    if !dry_run && !stats.issue_urls.is_empty() {
        println!("\nCreated issues:");
        for url in &stats.issue_urls {
            println!("   {}", url);
        }
    }

    println!("{}", "=".repeat(80));

    Ok(())
}
