//! Agent detection functions.

use crate::detection::{check_version, find_executable, parse_version};
use crate::{AgentKind, AgentStatus, DetectionError, InstalledMetadata};
use futures::future::join_all;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

/// Detect a single agent by kind.
///
/// This function checks if the specified agent is installed and usable.
/// It searches the system PATH for the agent's executable and verifies
/// its availability by running `--version` with a 2-second timeout.
///
/// # Detection Process
///
/// 1. Search for executable in PATH and fallback locations
/// 2. Run `{executable} --version` with 2-second timeout
/// 3. Parse semantic version from output using regex
/// 4. Return `Installed` with metadata if all steps succeed
///
/// # Arguments
///
/// * `kind` - The type of agent to detect
///
/// # Returns
///
/// An `AgentStatus` representing the detection result:
/// - `Installed(metadata)` - Agent found and usable
/// - `NotInstalled` - Agent not found or version check timed out
/// - `VersionMismatch { .. }` - Agent found but version incompatible
/// - `Unknown { .. }` - Detection failed with error
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{AgentKind, AgentStatus, detect};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let status = detect(AgentKind::ClaudeCode).await;
///     if status.is_usable() {
///         println!("Claude Code is available at {:?}", status.path());
///     }
/// }
/// ```
pub async fn detect(kind: AgentKind) -> AgentStatus {
    // Step 1: Find executable in PATH or fallback locations
    let path = match find_executable(kind.executable_name()) {
        Some(p) => p,
        None => return AgentStatus::NotInstalled,
    };

    // Step 2: Check version with 2s timeout
    let version_output = match check_version(&path).await {
        Ok(output) => output,
        Err(DetectionError::Timeout) => return AgentStatus::NotInstalled,
        Err(e) => {
            return AgentStatus::Unknown {
                error: e.clone(),
                message: format!(
                    "Failed to verify {}: {}",
                    kind.display_name(),
                    e.description()
                ),
            }
        }
    };

    // Step 3: Parse version from output
    let version = match parse_version(&version_output) {
        Ok(v) => v,
        Err(e) => {
            return AgentStatus::Unknown {
                error: e,
                message: format!("Failed to parse version from: {}", version_output.trim()),
            }
        }
    };

    // Step 4: Build metadata and return Installed
    AgentStatus::Installed(InstalledMetadata {
        path: path.clone(),
        version,
        install_method: detect_install_method(&path),
        last_verified: SystemTime::now(),
        reasoning_level: None,
    })
}

/// Detect the installation method from the executable path.
///
/// This heuristic checks the path for common patterns that indicate
/// how the tool was installed.
fn detect_install_method(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    if path_str.contains(".npm") || path_str.contains("node_modules") {
        Some("npm".to_string())
    } else if path_str.contains(".cargo") {
        Some("cargo".to_string())
    } else if path_str.contains("homebrew") || path_str.contains("linuxbrew") {
        Some("brew".to_string())
    } else if path_str.contains("mise") {
        Some("mise".to_string())
    } else {
        None
    }
}

/// Detect all known agents in parallel.
///
/// This function detects all agents defined in `AgentKind` concurrently,
/// returning a map of agent kinds to their detection status.
///
/// # Performance
///
/// Detection is performed in parallel using `futures::future::join_all`,
/// so the total detection time is approximately the time of the slowest
/// agent detection, not the sum of all detection times.
///
/// # Returns
///
/// A `HashMap` mapping each `AgentKind` to its `AgentStatus`.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{AgentKind, detect_all};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let all = detect_all().await;
///
///     for (kind, status) in &all {
///         println!("{}: {}", kind.display_name(),
///             if status.is_usable() { "available" } else { "not available" });
///     }
///
///     // Check specific agent
///     if let Some(status) = all.get(&AgentKind::ClaudeCode) {
///         if status.is_usable() {
///             println!("Claude Code ready!");
///         }
///     }
/// }
/// ```
pub async fn detect_all() -> HashMap<AgentKind, AgentStatus> {
    let futures: Vec<_> = AgentKind::all()
        .map(|kind| async move { (kind, detect(kind).await) })
        .collect();

    join_all(futures).await.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_all_returns_all_agents() {
        let all = detect_all().await;

        // Should have an entry for each agent kind
        assert_eq!(all.len(), 4);
        assert!(all.contains_key(&AgentKind::ClaudeCode));
        assert!(all.contains_key(&AgentKind::Codex));
        assert!(all.contains_key(&AgentKind::OpenCode));
        assert!(all.contains_key(&AgentKind::Gemini));
    }

    #[tokio::test]
    async fn test_detect_all_parallel_execution() {
        // This test verifies the function completes (parallel execution works)
        // Actual parallel timing would require real I/O
        let all = detect_all().await;
        assert!(!all.is_empty());
    }

    #[test]
    fn test_detect_install_method_npm() {
        let path = std::path::PathBuf::from("/home/user/.npm-global/bin/opencode");
        assert_eq!(detect_install_method(&path), Some("npm".to_string()));

        let path = std::path::PathBuf::from("/usr/local/lib/node_modules/.bin/tool");
        assert_eq!(detect_install_method(&path), Some("npm".to_string()));
    }

    #[test]
    fn test_detect_install_method_cargo() {
        let path = std::path::PathBuf::from("/home/user/.cargo/bin/tool");
        assert_eq!(detect_install_method(&path), Some("cargo".to_string()));
    }

    #[test]
    fn test_detect_install_method_brew() {
        let path = std::path::PathBuf::from("/home/linuxbrew/.linuxbrew/bin/tool");
        assert_eq!(detect_install_method(&path), Some("brew".to_string()));

        let path = std::path::PathBuf::from("/opt/homebrew/bin/tool");
        assert_eq!(detect_install_method(&path), Some("brew".to_string()));
    }

    #[test]
    fn test_detect_install_method_mise() {
        let path =
            std::path::PathBuf::from("/home/user/.local/share/mise/installs/tool/bin/binary");
        assert_eq!(detect_install_method(&path), Some("mise".to_string()));
    }

    #[test]
    fn test_detect_install_method_unknown() {
        let path = std::path::PathBuf::from("/usr/bin/tool");
        assert_eq!(detect_install_method(&path), None);
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::detection::{check_version, find_executable, parse_version};

    // Unit tests for synchronous functions - these are deterministic and stable
    #[test]
    fn test_find_executable_returns_none_for_nonexistent() {
        let result = find_executable("definitely_not_a_real_agent_cli_xyz123");
        assert!(result.is_none());
    }

    #[test]
    fn test_version_parsing_claude_format() {
        let version = parse_version("2.1.12 (Claude Code)").unwrap();
        assert_eq!(version.to_string(), "2.1.12");
    }

    #[test]
    fn test_version_parsing_codex_format() {
        let version = parse_version("codex-cli 0.87.0").unwrap();
        assert_eq!(version.to_string(), "0.87.0");
    }

    #[test]
    fn test_version_parsing_opencode_format() {
        let version = parse_version("1.1.25").unwrap();
        assert_eq!(version.to_string(), "1.1.25");
    }

    #[test]
    fn test_version_parsing_gemini_format() {
        let version = parse_version("gemini 0.1.5").unwrap();
        assert_eq!(version.to_string(), "0.1.5");
    }

    #[test]
    fn test_version_parsing_invalid() {
        let result = parse_version("no version here");
        assert!(matches!(result, Err(DetectionError::VersionParseFailed)));
    }

    // Async tests for check_version with real system executables
    #[tokio::test(flavor = "current_thread")]
    async fn test_check_version_io_error_for_nonexistent() {
        let exec_path = std::path::PathBuf::from("/nonexistent/path/to/agent");
        let result = check_version(&exec_path).await;
        assert!(matches!(result, Err(DetectionError::IoError)));
    }
}
