/// Common pattern utilities for key detection
pub struct PatternUtils;

impl PatternUtils {
    /// Check if a string has minimum entropy (helps filter false positives)
    pub fn has_min_entropy(s: &str, min_entropy: f64) -> bool {
        let entropy = Self::calculate_entropy(s);
        entropy >= min_entropy
    }

    /// Calculate Shannon entropy of a string
    pub fn calculate_entropy(s: &str) -> f64 {
        use std::collections::HashMap;

        if s.is_empty() {
            return 0.0;
        }

        let mut char_counts = HashMap::new();
        for c in s.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let len = s.len() as f64;
        let mut entropy = 0.0;

        for count in char_counts.values() {
            let p = (*count as f64) / len;
            entropy -= p * p.log2();
        }

        entropy
    }

    /// Check if string has mixed case (upper and lower)
    pub fn has_mixed_case(s: &str) -> bool {
        let has_upper = s.chars().any(|c| c.is_uppercase());
        let has_lower = s.chars().any(|c| c.is_lowercase());
        has_upper && has_lower
    }

    /// Check if string has digits
    pub fn has_digits(s: &str) -> bool {
        s.chars().any(|c| c.is_ascii_digit())
    }

    /// Check if string has letters
    pub fn has_letters(s: &str) -> bool {
        s.chars().any(|c| c.is_alphabetic())
    }

    /// Check if string looks like a hash (all hex, common hash lengths)
    pub fn looks_like_hash(s: &str) -> bool {
        // Common hash lengths: MD5=32, SHA1=40, SHA256=64
        let common_hash_lengths = [32, 40, 64];
        let is_hex = s.chars().all(|c| c.is_ascii_hexdigit());
        is_hex && common_hash_lengths.contains(&s.len())
    }

    /// Extract line number and context from content
    pub fn get_line_context(content: &str, match_pos: usize, context_lines: usize) -> (usize, String) {
        let before_match = &content[..match_pos];
        let line_number = before_match.lines().count();

        let lines: Vec<&str> = content.lines().collect();
        let start = line_number.saturating_sub(context_lines);
        let end = (line_number + context_lines + 1).min(lines.len());

        let context = lines[start..end].join("\n");

        (line_number, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        // Low entropy (all same character)
        assert!(PatternUtils::calculate_entropy("aaaaaaa") < 1.0);

        // High entropy (random-looking)
        assert!(PatternUtils::calculate_entropy("aB3xY9zQ2m") > 3.0);
    }

    #[test]
    fn test_mixed_case() {
        assert!(PatternUtils::has_mixed_case("AbCdEf"));
        assert!(!PatternUtils::has_mixed_case("abcdef"));
        assert!(!PatternUtils::has_mixed_case("ABCDEF"));
    }

    #[test]
    fn test_has_digits() {
        assert!(PatternUtils::has_digits("abc123"));
        assert!(!PatternUtils::has_digits("abcdef"));
    }

    #[test]
    fn test_looks_like_hash() {
        // MD5 (32 hex chars)
        assert!(PatternUtils::looks_like_hash("5d41402abc4b2a76b9719d911017c592"));

        // SHA1 (40 hex chars)
        assert!(PatternUtils::looks_like_hash("356a192b7913b04c54574d18c28d46e6395428ab"));

        // Not a hash (has non-hex)
        assert!(!PatternUtils::looks_like_hash("5d41402abc4b2a76b9719d911017c59g"));

        // Not a hash (wrong length)
        assert!(!PatternUtils::looks_like_hash("5d41402abc4b2a76"));
    }

    #[test]
    fn test_get_line_context() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        let match_pos = content.find("line 3").unwrap();

        let (line_num, context) = PatternUtils::get_line_context(content, match_pos, 1);

        assert_eq!(line_num, 2); // 0-indexed
        assert!(context.contains("line 2"));
        assert!(context.contains("line 3"));
        assert!(context.contains("line 4"));
    }
}
