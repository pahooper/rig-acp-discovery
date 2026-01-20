//! Version output parsing with regex extraction.

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
/// - `v1.2.3` -> 1.2.3 (strips 'v' prefix)
/// - `1.2` -> 1.2.0 (appends .0 for 2-part versions)
/// - `v0.24.4` -> 0.24.4 (Gemini CLI format)
///
/// # Arguments
///
/// * `output` - The CLI output text to parse
///
/// # Returns
///
/// `Some((version, raw_match))` where:
/// - `version` is the parsed semantic version
/// - `raw_match` is the matched substring from the output (e.g., "v1.2.3", "1.2")
///
/// Returns `None` if no version pattern matches or the matched string
/// cannot be parsed as valid semver.
pub(crate) fn parse_version(output: &str) -> Option<(Version, String)> {
    // First try: 3-part version with optional 'v' prefix
    // Pattern: v?X.Y.Z where X, Y, Z are digits
    let re_3part = Regex::new(r"[vV]?(\d+)\.(\d+)\.(\d+)").expect("Invalid regex pattern");

    if let Some(caps) = re_3part.captures(output) {
        let raw_match = caps.get(0).expect("Capture group 0 should exist").as_str();
        // Strip 'v' or 'V' prefix for parsing
        let version_str = raw_match.trim_start_matches(['v', 'V']);

        if let Ok(version) = Version::parse(version_str) {
            return Some((version, raw_match.to_string()));
        }
    }

    // Second try: 2-part version with optional 'v' prefix
    // Pattern: v?X.Y where X, Y are digits
    // We use a simpler pattern and check manually that it's not part of a 3-part version
    let re_2part = Regex::new(r"[vV]?(\d+)\.(\d+)").expect("Invalid regex pattern");

    if let Some(caps) = re_2part.captures(output) {
        let raw_match = caps.get(0).expect("Capture group 0 should exist").as_str();
        let match_end = caps.get(0).expect("Capture group 0 should exist").end();

        // Check if this is followed by another .digit (would be part of 3-part version)
        // If so, skip this match as it was already handled above (or should have been)
        let remaining = &output[match_end..];
        if remaining.starts_with('.')
            && remaining.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
        {
            // This is part of a 3-part version, but we didn't match it above
            // This shouldn't happen normally, but handle it gracefully
            return None;
        }

        // Strip 'v' or 'V' prefix and append .0 for semver compatibility
        let version_str = raw_match.trim_start_matches(['v', 'V']);
        let version_str_with_patch = format!("{}.0", version_str);

        if let Ok(version) = Version::parse(&version_str_with_patch) {
            return Some((version, raw_match.to_string()));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_code_version() {
        let output = "2.1.12 (Claude Code)";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(2, 1, 12));
        assert_eq!(raw, "2.1.12");
    }

    #[test]
    fn test_parse_codex_version() {
        let output = "codex-cli 0.87.0";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(0, 87, 0));
        assert_eq!(raw, "0.87.0");
    }

    #[test]
    fn test_parse_opencode_version() {
        let output = "1.1.25";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(1, 1, 25));
        assert_eq!(raw, "1.1.25");
    }

    #[test]
    fn test_parse_version_with_newline() {
        let output = "tool version 3.2.1\n";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(3, 2, 1));
        assert_eq!(raw, "3.2.1");
    }

    #[test]
    fn test_parse_version_multiline() {
        let output = "My Tool\nVersion: 1.0.0\nBuilt on 2025-01-01";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(1, 0, 0));
        assert_eq!(raw, "1.0.0");
    }

    #[test]
    fn test_parse_version_no_match() {
        let output = "no version here";
        let result = parse_version(output);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_version_v_prefix() {
        let output = "v1.2.3";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
        assert_eq!(raw, "v1.2.3");
    }

    #[test]
    fn test_parse_version_uppercase_v_prefix() {
        let output = "V2.0.0";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(2, 0, 0));
        assert_eq!(raw, "V2.0.0");
    }

    #[test]
    fn test_parse_version_two_part() {
        let output = "version 1.2";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(1, 2, 0));
        assert_eq!(raw, "1.2");
    }

    #[test]
    fn test_parse_version_two_part_v_prefix() {
        let output = "v1.2 beta";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(1, 2, 0));
        assert_eq!(raw, "v1.2");
    }

    #[test]
    fn test_parse_version_gemini_format() {
        let output = "gemini v0.24.4";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(0, 24, 4));
        assert_eq!(raw, "v0.24.4");
    }

    #[test]
    fn test_parse_version_prefers_3part_over_2part() {
        // When both 2-part and 3-part patterns could match,
        // the 3-part should be preferred
        let output = "version 1.2.3 is available";
        let (version, raw) = parse_version(output).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
        assert_eq!(raw, "1.2.3");
    }
}
