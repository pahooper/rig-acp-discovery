//! Version output parsing with regex extraction.

use crate::DetectionError;
use regex::Regex;
use semver::Version;

/// Parse a semantic version from CLI output.
///
/// This function extracts a semantic version (major.minor.patch) from
/// arbitrary CLI output text. It handles various output formats:
///
/// - `2.1.12 (Claude Code)` -> 2.1.12
/// - `codex-cli 0.87.0` -> 0.87.0
/// - `1.1.25` -> 1.1.25
///
/// # Arguments
///
/// * `output` - The CLI output text to parse
///
/// # Returns
///
/// `Ok(Version)` if a valid semantic version is found,
/// `Err(DetectionError::VersionParseFailed)` if no version pattern matches
/// or the matched string is not a valid semver.
pub(crate) fn parse_version(output: &str) -> Result<Version, DetectionError> {
    // Regex to match semantic version pattern: major.minor.patch
    // This handles versions embedded in various output formats
    let re = Regex::new(r"(\d+)\.(\d+)\.(\d+)").expect("Invalid regex pattern");

    if let Some(caps) = re.captures(output) {
        let version_str = caps.get(0).expect("Capture group 0 should exist").as_str();
        Version::parse(version_str).map_err(|_| DetectionError::VersionParseFailed)
    } else {
        Err(DetectionError::VersionParseFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_code_version() {
        let output = "2.1.12 (Claude Code)";
        let result = parse_version(output).unwrap();
        assert_eq!(result, Version::new(2, 1, 12));
    }

    #[test]
    fn test_parse_codex_version() {
        let output = "codex-cli 0.87.0";
        let result = parse_version(output).unwrap();
        assert_eq!(result, Version::new(0, 87, 0));
    }

    #[test]
    fn test_parse_opencode_version() {
        let output = "1.1.25";
        let result = parse_version(output).unwrap();
        assert_eq!(result, Version::new(1, 1, 25));
    }

    #[test]
    fn test_parse_version_with_newline() {
        let output = "tool version 3.2.1\n";
        let result = parse_version(output).unwrap();
        assert_eq!(result, Version::new(3, 2, 1));
    }

    #[test]
    fn test_parse_version_multiline() {
        let output = "My Tool\nVersion: 1.0.0\nBuilt on 2025-01-01";
        let result = parse_version(output).unwrap();
        assert_eq!(result, Version::new(1, 0, 0));
    }

    #[test]
    fn test_parse_version_no_match() {
        let output = "no version here";
        let result = parse_version(output);
        assert!(matches!(result, Err(DetectionError::VersionParseFailed)));
    }

    #[test]
    fn test_parse_version_incomplete() {
        let output = "version 1.2";
        let result = parse_version(output);
        assert!(matches!(result, Err(DetectionError::VersionParseFailed)));
    }
}
