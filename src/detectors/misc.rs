use crate::core::results::DetectedKey;
use crate::core::traits::KeyDetector;
use crate::utils::PatternUtils;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Private Keys
    static ref RSA_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN RSA PRIVATE KEY-----").unwrap();
    static ref SSH_DSA_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN DSA PRIVATE KEY-----").unwrap();
    static ref SSH_EC_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN EC PRIVATE KEY-----").unwrap();
    static ref PGP_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN PGP PRIVATE KEY BLOCK-----").unwrap();

    // Payment Services
    static ref PAYPAL_BRAINTREE: Regex = Regex::new(r"access_token\$production\$[0-9a-z]{16}\$[0-9a-f]{32}").unwrap();
    static ref SQUARE_ACCESS: Regex = Regex::new(r"sq0atp-[0-9A-Za-z\-_]{22}").unwrap();
    static ref SQUARE_OAUTH: Regex = Regex::new(r"sq0csp-[0-9A-Za-z\-_]{43}").unwrap();
    static ref PICATIC: Regex = Regex::new(r"sk_live_[0-9a-z]{32}").unwrap();

    // Communication Services
    static ref TELEGRAM_BOT: Regex = Regex::new(r"[0-9]+:AA[0-9A-Za-z\-_]{33}").unwrap();
    static ref TWILIO_API: Regex = Regex::new(r"SK[0-9a-fA-F]{32}").unwrap();
    static ref MAILCHIMP: Regex = Regex::new(r"[0-9a-f]{32}-us[0-9]{1,2}").unwrap();
    static ref MAILGUN: Regex = Regex::new(r"key-[0-9a-zA-Z]{32}").unwrap();

    // Social Media
    static ref TWITTER_ACCESS: Regex = Regex::new(r"[tT][wW][iI][tT][tT][eE][rR].*[1-9][0-9]+-[0-9a-zA-Z]{40}").unwrap();
    static ref TWITTER_OAUTH: Regex = Regex::new(r#"[tT][wW][iI][tT][tT][eE][rR].*['|"][0-9a-zA-Z]{35,44}['|"]"#).unwrap();
    static ref FACEBOOK_ACCESS: Regex = Regex::new(r"EAACEdEose0cBA[0-9A-Za-z]+").unwrap();
    static ref FACEBOOK_OAUTH: Regex = Regex::new(r#"[fF][aA][cC][eE][bB][oO][oO][kK].*['|"][0-9a-f]{32}['|"]"#).unwrap();

    // Cloud Services
    static ref HEROKU_API: Regex = Regex::new(r"[hH][eE][rR][oO][kK][uU].*[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}").unwrap();
    static ref AMAZON_MWS: Regex = Regex::new(r"amzn\.mws\.[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();

    // Generic Patterns
    static ref PASSWORD_IN_URL: Regex = Regex::new(r#"[a-zA-Z]{3,10}://[^/\s:@]{3,20}:[^/\s:@]{3,20}@.{1,100}["'\s]"#).unwrap();
    static ref GENERIC_API_KEY: Regex = Regex::new(r#"[aA][pP][iI]_?[kK][eE][yY].*['|""][0-9a-zA-Z]{32,45}['|""]"#).unwrap();
    static ref GENERIC_SECRET: Regex = Regex::new(r#"[sS][eE][cC][rR][eE][tT].*['|""][0-9a-zA-Z]{32,45}['|""]"#).unwrap();
}

pub struct MiscDetector {
    patterns: Vec<Regex>,
}

impl MiscDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Private keys
                RSA_PRIVATE_KEY.clone(),
                SSH_DSA_PRIVATE_KEY.clone(),
                SSH_EC_PRIVATE_KEY.clone(),
                PGP_PRIVATE_KEY.clone(),
                // Payment
                PAYPAL_BRAINTREE.clone(),
                SQUARE_ACCESS.clone(),
                SQUARE_OAUTH.clone(),
                PICATIC.clone(),
                // Communication
                TELEGRAM_BOT.clone(),
                TWILIO_API.clone(),
                MAILCHIMP.clone(),
                MAILGUN.clone(),
                // Social
                TWITTER_ACCESS.clone(),
                TWITTER_OAUTH.clone(),
                FACEBOOK_ACCESS.clone(),
                FACEBOOK_OAUTH.clone(),
                // Cloud
                HEROKU_API.clone(),
                AMAZON_MWS.clone(),
                // Generic
                PASSWORD_IN_URL.clone(),
                GENERIC_API_KEY.clone(),
                GENERIC_SECRET.clone(),
            ],
        }
    }
}

impl Default for MiscDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDetector for MiscDetector {
    fn name(&self) -> &str {
        "misc"
    }

    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.find_iter(content) {
                let key = capture.as_str();
                let (line_number, context) =
                    PatternUtils::get_line_context(content, capture.start(), 2);

                // Determine more specific key type based on pattern
                let key_type = if key.contains("RSA PRIVATE KEY") {
                    "rsa_private_key"
                } else if key.contains("DSA PRIVATE KEY") {
                    "ssh_dsa_private_key"
                } else if key.contains("EC PRIVATE KEY") {
                    "ssh_ec_private_key"
                } else if key.contains("PGP PRIVATE KEY") {
                    "pgp_private_key"
                } else if key.contains("access_token$production") {
                    "paypal_braintree"
                } else if key.starts_with("sq0atp-") {
                    "square_access"
                } else if key.starts_with("sq0csp-") {
                    "square_oauth"
                } else if key.contains(":AA") {
                    "telegram_bot"
                } else if key.starts_with("SK") && key.len() == 34 {
                    "twilio"
                } else if key.contains("-us") {
                    "mailchimp"
                } else if key.starts_with("key-") {
                    "mailgun"
                } else if key.to_lowercase().contains("twitter") {
                    "twitter"
                } else if key.to_lowercase().contains("facebook") {
                    "facebook"
                } else if key.to_lowercase().contains("heroku") {
                    "heroku"
                } else if key.contains("amzn.mws") {
                    "amazon_mws"
                } else if key.contains("://") {
                    "password_in_url"
                } else if key.to_lowercase().contains("api") {
                    "generic_api_key"
                } else {
                    "generic_secret"
                };

                detected.push(DetectedKey {
                    key: key.to_string(),
                    key_type: key_type.to_string(),
                    repository: String::new(),
                    file_path: file_path.to_string(),
                    file_url: String::new(),
                    line_number: Some(line_number),
                    context: Some(context),
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
            "BEGIN RSA PRIVATE KEY".to_string(),
            "BEGIN DSA PRIVATE KEY".to_string(),
            "BEGIN EC PRIVATE KEY".to_string(),
            "BEGIN PGP PRIVATE KEY".to_string(),
            "TELEGRAM_BOT_TOKEN".to_string(),
            "TWILIO_API_KEY".to_string(),
            "MAILCHIMP_API_KEY".to_string(),
            "MAILGUN_API_KEY".to_string(),
            "TWITTER_API_KEY".to_string(),
            "FACEBOOK_APP_SECRET".to_string(),
            "HEROKU_API_KEY".to_string(),
            "API_KEY extension:env".to_string(),
            "SECRET extension:env".to_string(),
            "PASSWORD".to_string(),
        ]
    }

    fn file_extensions(&self) -> &[&str] {
        &[".env", ".py", ".js", ".json", ".yaml", ".yml", ".txt", ".config", ".sh", ".pem", ".key"]
    }
}
