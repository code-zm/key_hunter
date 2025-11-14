use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// GitHub Token pattern
    static ref GITHUB_TOKEN: Regex = Regex::new(r#"[gG][iI][tT][hH][uU][bB].*['|"][0-9a-zA-Z]{35,40}['|"]"#).unwrap();

    /// GitHub Personal Access Token (modern format)
    static ref GITHUB_PAT: Regex = Regex::new(r"ghp_[0-9a-zA-Z]{36}").unwrap();

    /// GitHub OAuth Token
    static ref GITHUB_OAUTH: Regex = Regex::new(r"gho_[0-9a-zA-Z]{36}").unwrap();

    /// GitHub App Token
    static ref GITHUB_APP: Regex = Regex::new(r"(ghu|ghs)_[0-9a-zA-Z]{36}").unwrap();

    /// GitHub Refresh Token
    static ref GITHUB_REFRESH: Regex = Regex::new(r"ghr_[0-9a-zA-Z]{36}").unwrap();
}

pub struct GitHubKeysDetector {
    patterns: Vec<Regex>,
}

impl GitHubKeysDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                GITHUB_TOKEN.clone(),
                GITHUB_PAT.clone(),
                GITHUB_OAUTH.clone(),
                GITHUB_APP.clone(),
                GITHUB_REFRESH.clone(),
            ],
        }
    }
}

impl Default for GitHubKeysDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for GitHubKeysDetector {
    fn name(&self) -> &str {
        "github"
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
                    key_type: "github".to_string(),
                    repository: String::new(),
                    file_path: file_path.to_string(),
                    file_url: String::new(),
                    line_number: Some(line_number),
                    context: Some(context),
                    repo_owner_email: None,
                    commit_author_email: None,
                    commit_sha: None,
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
            "GITHUB_TOKEN".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config", ".sh"]
    }
}
