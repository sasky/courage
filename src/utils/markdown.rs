//! Markdown parsing utilities for relationship extraction

use regex::Regex;

/// Pattern for extracting relationship values from markdown
/// Matches **Label:** followed by optional spaces (not newlines) and content until end of line
const RELATIONSHIP_PATTERN: &str = r"\*\*{}:\*\*[ \t]*([^\n]*)";

/// Build a regex pattern for extracting a labeled relationship value
pub fn relationship_pattern(label: &str) -> Regex {
    let pattern = RELATIONSHIP_PATTERN.replace("{}", &regex::escape(label));
    Regex::new(&pattern).expect("Invalid regex pattern")
}

/// Extract value after **Label:** pattern from text
pub fn extract_labeled_value(text: &str, label: &str) -> String {
    let re = relationship_pattern(label);
    re.captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_labeled_value_basic() {
        let text = "**Parents:** [[Mom]], [[Dad]]\n**Children:** \n";
        assert_eq!(extract_labeled_value(text, "Parents"), "[[Mom]], [[Dad]]");
        assert_eq!(extract_labeled_value(text, "Children"), "");
    }

    #[test]
    fn test_extract_labeled_value_with_spaces() {
        let text = "**Friends:**   [[Alice]], [[Bob]]  \n";
        assert_eq!(extract_labeled_value(text, "Friends"), "[[Alice]], [[Bob]]");
    }

    #[test]
    fn test_extract_labeled_value_missing_label() {
        let text = "**Parents:** [[Mom]]\n";
        assert_eq!(extract_labeled_value(text, "Siblings"), "");
    }

    #[test]
    fn test_relationship_pattern_escapes_special_chars() {
        // Labels with special regex characters should be escaped
        let re = relationship_pattern("Test (Label)");
        assert!(re.is_match("**Test (Label):** value"));
    }
}
