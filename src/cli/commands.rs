use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "key-hunter")]
#[command(version, about = "A modular framework for hunting API keys in code repositories", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for exposed keys
    Search {
        /// Search provider to use (github, gitlab, local)
        #[arg(short, long, default_value = "github")]
        provider: String,

        /// Key type to search for (shodan, aws, github, all)
        #[arg(short, long, default_value = "all")]
        key_type: String,

        /// Custom search query (overrides default queries)
        #[arg(short, long)]
        query: Option<String>,

        /// Output file for results (default: results/<key-type>/valid_keys_<timestamp>.json)
        #[arg(short, long)]
        output: Option<String>,

        /// Validate keys immediately as they're found
        #[arg(long)]
        validate: bool,
    },

    /// Validate keys from a file
    Validate {
        /// Input file containing keys to validate
        #[arg(short, long)]
        input: String,

        /// Output file for validation results
        #[arg(short, long, default_value = "validated_results.json")]
        output: String,

        /// Key type (shodan, aws, github, all)
        #[arg(short, long, default_value = "all")]
        key_type: String,
    },

    /// Test a single key
    Test {
        /// The key to test
        key: String,

        /// Key type (shodan, aws, github)
        #[arg(short, long)]
        key_type: String,
    },

    /// List available detectors and validators
    List {
        /// What to list (detectors, validators, all)
        #[arg(default_value = "all")]
        what: String,
    },

    /// Create GitHub issues for exposed keys
    Report {
        /// Directory containing result files (default: results/)
        #[arg(short, long, default_value = "results")]
        results_dir: String,

        /// Filter by key type (shodan, claude, openai, all)
        #[arg(short, long, default_value = "all")]
        key_type: String,

        /// Dry run - print issues without creating them
        #[arg(long)]
        dry_run: bool,
    },
}
