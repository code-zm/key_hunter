use key_hunter::core::config::ValidatorsConfig;
use key_hunter::validators;
use std::time::Duration;

#[test]
fn test_validators_use_custom_config_rate_limits() {
    // Create a custom config with different rate limits
    let custom_config = ValidatorsConfig {
        openai_rate_limit_ms: 5000,      // 5 seconds
        claude_rate_limit_ms: 4000,      // 4 seconds
        gemini_rate_limit_ms: 3500,      // 3.5 seconds
        shodan_rate_limit_ms: 2500,      // 2.5 seconds
        xai_rate_limit_ms: 1500,         // 1.5 seconds
        openrouter_rate_limit_ms: 6000,  // 6 seconds
        github_rate_limit_ms: 3000,      // 3 seconds
    };

    // Get all validators with custom config
    let validators_map = validators::all_validators(&custom_config);

    // Verify each validator uses the custom rate limit
    let openai_validator = validators_map.get("openai").expect("OpenAI validator not found");
    assert_eq!(
        openai_validator.rate_limit(),
        Duration::from_millis(5000),
        "OpenAI validator should use custom 5000ms rate limit"
    );

    let claude_validator = validators_map.get("claude").expect("Claude validator not found");
    assert_eq!(
        claude_validator.rate_limit(),
        Duration::from_millis(4000),
        "Claude validator should use custom 4000ms rate limit"
    );

    let gemini_validator = validators_map.get("gemini").expect("Gemini validator not found");
    assert_eq!(
        gemini_validator.rate_limit(),
        Duration::from_millis(3500),
        "Gemini validator should use custom 3500ms rate limit"
    );

    let shodan_validator = validators_map.get("shodan").expect("Shodan validator not found");
    assert_eq!(
        shodan_validator.rate_limit(),
        Duration::from_millis(2500),
        "Shodan validator should use custom 2500ms rate limit"
    );

    let xai_validator = validators_map.get("xai").expect("xAI validator not found");
    assert_eq!(
        xai_validator.rate_limit(),
        Duration::from_millis(1500),
        "xAI validator should use custom 1500ms rate limit"
    );

    let openrouter_validator = validators_map.get("openrouter").expect("OpenRouter validator not found");
    assert_eq!(
        openrouter_validator.rate_limit(),
        Duration::from_millis(6000),
        "OpenRouter validator should use custom 6000ms rate limit"
    );

    let github_validator = validators_map.get("github").expect("GitHub validator not found");
    assert_eq!(
        github_validator.rate_limit(),
        Duration::from_millis(3000),
        "GitHub validator should use custom 3000ms rate limit"
    );
}

#[test]
fn test_validators_use_default_rate_limits() {
    // Use default config
    let default_config = ValidatorsConfig::default();

    // Get all validators with default config
    let validators_map = validators::all_validators(&default_config);

    // Verify each validator uses the default rate limit
    let openai_validator = validators_map.get("openai").expect("OpenAI validator not found");
    assert_eq!(
        openai_validator.rate_limit(),
        Duration::from_millis(1000),
        "OpenAI validator should use default 1000ms rate limit"
    );

    let claude_validator = validators_map.get("claude").expect("Claude validator not found");
    assert_eq!(
        claude_validator.rate_limit(),
        Duration::from_millis(2000),
        "Claude validator should use default 2000ms rate limit"
    );

    let gemini_validator = validators_map.get("gemini").expect("Gemini validator not found");
    assert_eq!(
        gemini_validator.rate_limit(),
        Duration::from_millis(2000),
        "Gemini validator should use default 2000ms rate limit"
    );

    let shodan_validator = validators_map.get("shodan").expect("Shodan validator not found");
    assert_eq!(
        shodan_validator.rate_limit(),
        Duration::from_millis(1000),
        "Shodan validator should use default 1000ms rate limit"
    );

    let xai_validator = validators_map.get("xai").expect("xAI validator not found");
    assert_eq!(
        xai_validator.rate_limit(),
        Duration::from_millis(1000),
        "xAI validator should use default 1000ms rate limit"
    );

    let openrouter_validator = validators_map.get("openrouter").expect("OpenRouter validator not found");
    assert_eq!(
        openrouter_validator.rate_limit(),
        Duration::from_millis(3000),
        "OpenRouter validator should use default 3000ms rate limit"
    );

    let github_validator = validators_map.get("github").expect("GitHub validator not found");
    assert_eq!(
        github_validator.rate_limit(),
        Duration::from_millis(2000),
        "GitHub validator should use default 2000ms rate limit"
    );
}

#[test]
fn test_get_validator_by_type_uses_custom_config() {
    let custom_config = ValidatorsConfig {
        openai_rate_limit_ms: 7000,
        claude_rate_limit_ms: 8000,
        gemini_rate_limit_ms: 9000,
        shodan_rate_limit_ms: 10000,
        xai_rate_limit_ms: 11000,
        openrouter_rate_limit_ms: 12000,
        github_rate_limit_ms: 13000,
    };

    // Test getting individual validators by type
    let openai = validators::get_validator("openai", &custom_config).expect("Should get OpenAI validator");
    assert_eq!(openai.rate_limit(), Duration::from_millis(7000));

    let claude = validators::get_validator("claude", &custom_config).expect("Should get Claude validator");
    assert_eq!(claude.rate_limit(), Duration::from_millis(8000));

    let gemini = validators::get_validator("gemini", &custom_config).expect("Should get Gemini validator");
    assert_eq!(gemini.rate_limit(), Duration::from_millis(9000));

    let shodan = validators::get_validator("shodan", &custom_config).expect("Should get Shodan validator");
    assert_eq!(shodan.rate_limit(), Duration::from_millis(10000));

    let xai = validators::get_validator("xai", &custom_config).expect("Should get xAI validator");
    assert_eq!(xai.rate_limit(), Duration::from_millis(11000));

    let openrouter = validators::get_validator("openrouter", &custom_config).expect("Should get OpenRouter validator");
    assert_eq!(openrouter.rate_limit(), Duration::from_millis(12000));

    let github = validators::get_validator("github", &custom_config).expect("Should get GitHub validator");
    assert_eq!(github.rate_limit(), Duration::from_millis(13000));
}
