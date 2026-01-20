//! Prerequisite checking for agent installation.
//!
//! This module provides the [`can_install`] function for pre-flight checks
//! before attempting to install an agent.

use crate::{AgentKind, InstallError};
use regex::Regex;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Default timeout for prerequisite checks.
const PREREQ_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Check if prerequisites are met for installing the given agent.
///
/// This performs a pre-flight check before installation:
/// 1. Verifies the agent is supported on this platform
/// 2. Checks each prerequisite's check_command
/// 3. Parses version output and compares to minimum requirement
///
/// Returns `Ok(())` if installation can proceed, or an [`InstallError`]
/// with an actionable fix suggestion if not.
///
/// # Example
///
/// ```rust,no_run
/// use rig_acp_discovery::{AgentKind, can_install};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     // Check if we can install Codex (requires Node.js 18+)
///     match can_install(AgentKind::Codex).await {
///         Ok(()) => println!("Prerequisites met, ready to install"),
///         Err(e) => {
///             eprintln!("Cannot install: {}", e);
///             eprintln!("To fix: {}", e.fix_suggestion());
///         }
///     }
///
///     // Claude Code has no prerequisites (native installer)
///     assert!(can_install(AgentKind::ClaudeCode).await.is_ok());
/// }
/// ```
pub async fn can_install(kind: AgentKind) -> Result<(), InstallError> {
    let info = kind.install_info();

    // Check platform support
    if !info.is_supported {
        return Err(InstallError::UnsupportedPlatform {
            agent: kind,
            fix: format!("See {} for supported platforms", info.docs_url),
        });
    }

    // Check each prerequisite
    for prereq in &info.prerequisites {
        check_prerequisite(prereq).await?;
    }

    Ok(())
}

/// Check a single prerequisite.
///
/// Runs the check_command and verifies the version meets the minimum requirement.
async fn check_prerequisite(prereq: &crate::Prerequisite) -> Result<(), InstallError> {
    let check_command = match &prereq.check_command {
        Some(cmd) => cmd,
        None => return Ok(()), // No check command means we can't verify, assume OK
    };

    // Parse the check command (e.g., "node --version")
    let parts: Vec<&str> = check_command.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(()); // Empty command, assume OK
    }

    let program = parts[0];
    let args = &parts[1..];

    // Run the command with timeout
    let mut cmd = Command::new(program);
    cmd.args(args).kill_on_drop(true);

    let output = match timeout(PREREQ_CHECK_TIMEOUT, cmd.output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(_)) | Err(_) => {
            // Command failed or timed out - prerequisite is missing
            return Err(InstallError::PrerequisiteMissing {
                name: prereq.name.clone(),
                install_url: prereq.install_url.clone(),
                fix: format!(
                    "Install {} from {}",
                    prereq.name,
                    prereq
                        .install_url
                        .as_deref()
                        .unwrap_or("the official website")
                ),
            });
        }
    };

    // Get output (prefer stdout, fall back to stderr)
    let output_str = if !output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    // Parse version from output using regex
    let version_re = Regex::new(r"v?(\d+)\.(\d+)").expect("Invalid version regex");
    let (found_major, found_minor) = match version_re.captures(&output_str) {
        Some(caps) => {
            let major: u32 = caps
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            let minor: u32 = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            (major, minor)
        }
        None => {
            // Can't parse version - treat as missing (conservative approach)
            return Err(InstallError::PrerequisiteMissing {
                name: prereq.name.clone(),
                install_url: prereq.install_url.clone(),
                fix: format!(
                    "Install {} from {}",
                    prereq.name,
                    prereq
                        .install_url
                        .as_deref()
                        .unwrap_or("the official website")
                ),
            });
        }
    };

    // Extract minimum version from prereq name (e.g., "Node.js 18+" -> 18)
    let min_version_re = Regex::new(r"(\d+)\+").expect("Invalid min version regex");
    let required_major: u32 = min_version_re
        .captures(&prereq.name)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    // Compare versions
    if found_major < required_major {
        return Err(InstallError::PrerequisiteVersionMismatch {
            name: prereq.name.clone(),
            required: format!("{}+", required_major),
            found: format!("{}.{}", found_major, found_minor),
            fix: format!("Upgrade {} to version {}+", prereq.name, required_major),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InstallOptions;

    #[tokio::test]
    async fn test_can_install_claude_no_prereqs() {
        // Claude Code has no prerequisites (native installer), should always return Ok
        let result = can_install(AgentKind::ClaudeCode).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_can_install_opencode_no_prereqs() {
        // OpenCode primary method has no prerequisites (native installer)
        let result = can_install(AgentKind::OpenCode).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_can_install_codex_checks_nodejs() {
        // Codex requires Node.js 18+
        // This test will pass or fail based on system state, which is intentional
        let result = can_install(AgentKind::Codex).await;

        // Either Ok (Node.js installed and meets version) or appropriate error
        match result {
            Ok(()) => {
                // Node.js is installed and meets version requirement
            }
            Err(InstallError::PrerequisiteMissing { name, .. }) => {
                assert!(name.contains("Node.js"));
            }
            Err(InstallError::PrerequisiteVersionMismatch { name, .. }) => {
                assert!(name.contains("Node.js"));
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_can_install_gemini_checks_nodejs() {
        // Gemini requires Node.js 20+
        let result = can_install(AgentKind::Gemini).await;

        // Either Ok (Node.js 20+ installed) or appropriate error
        match result {
            Ok(()) => {
                // Node.js 20+ is installed
            }
            Err(InstallError::PrerequisiteMissing { name, .. }) => {
                assert!(name.contains("Node.js"));
            }
            Err(InstallError::PrerequisiteVersionMismatch { name, .. }) => {
                assert!(name.contains("Node.js"));
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[test]
    fn test_install_options_default() {
        let opts = InstallOptions::default();
        assert_eq!(opts.timeout, Duration::from_secs(300));
    }
}
