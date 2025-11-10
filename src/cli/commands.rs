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

        /// Maximum results per query (GitHub max is 1000)
        #[arg(short, long, default_value = "1000")]
        max_results: usize,

        /// Number of queries to run (if using default queries)
        #[arg(short = 'n', long, default_value = "3")]
        num_queries: usize,

        /// GitHub token for authenticated requests (can also use GITHUB_TOKEN env var)
        #[arg(long)]
        github_token: Option<String>,

        /// Output file for results (default: results/<key-type>/valid_keys_<timestamp>.json)
        #[arg(short, long)]
        output: Option<String>,

        /// Test keys immediately as they're found
        #[arg(short, long)]
        test_immediately: bool,

        /// Automatically split queries by file type/extension to get past GitHub's 1000 result limit
        #[arg(long)]
        auto_split: bool,
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

    /// Generate a report from results
    Report {
        /// Input file with results
        #[arg(short, long)]
        input: String,

        /// Output format (json, csv, html, text)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// List available detectors and validators
    List {
        /// What to list (detectors, validators, all)
        #[arg(default_value = "all")]
        what: String,
    },
}
