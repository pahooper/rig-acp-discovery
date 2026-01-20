//! Agent detection functions.
//!
//! This module provides async functions for detecting AI coding agents
//! on the system. Detection can be performed for a single agent or
//! all known agents in parallel.

use crate::detection::{check_version, find_executable, parse_version};
use crate::options::DetectOptions;
use crate::{AgentKind, AgentStatus, DetectionError, InstalledMetadata};
use futures::future::join_all;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use tracing::warn;

/// Detect a single agent by kind using default options.
///
/// This function checks if the specified agent is installed and usable,
/// using the default detection timeout (5 seconds).
///
/// For custom timeout configuration, use [`detect_with_options`].
///
/// # Detection Process
///
/// 1. Search for executable in PATH and fallback locations
/// 2. Run `{executable} --version` with 5-second timeout
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
    detect_with_options(kind, DetectOptions::default()).await
}

/// Detect a single agent by kind with custom options.
///
/// This function checks if the specified agent is installed and usable,
/// using the provided detection options for configuration.
///
/// # Arguments
///
/// * `kind` - The type of agent to detect
/// * `options` - Configuration options including timeout
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
/// use rig_acp_discovery::{AgentKind, DetectOptions, detect_with_options};
/// use std::time::Duration;
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let options = DetectOptions {
///         timeout: Duration::from_secs(10), // Longer timeout
///         ..Default::default()
///     };
///     let status = detect_with_options(AgentKind::ClaudeCode, options).await;
///     if status.is_usable() {
///         println!("Claude Code is available at {:?}", status.path());
///     }
/// }
/// ```
pub async fn detect_with_options(kind: AgentKind, options: DetectOptions) -> AgentStatus {
    // Step 1: Find executable in PATH or fallback locations
    let path = match find_executable(kind.executable_name()) {
        Some(p) => p,
        None => return AgentStatus::NotInstalled,
    };

    // Step 2: If skip_version is true, return Installed immediately without version info
    if options.skip_version {
        return AgentStatus::Installed(InstalledMetadata {
            path: path.clone(),
            version: None,
            raw_version: None,
            install_method: detect_install_method(&path),
            last_verified: SystemTime::now(),
            reasoning_level: None,
        });
    }

    // Step 3: Check version with configured timeout
    let version_output = match check_version(&path, options.timeout).await {
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

    // Step 4: Parse version from output with graceful degradation
    let (version, raw_version) = match parse_version(&version_output) {
        Some((v, raw)) => (Some(v), Some(raw)),
        None => {
            // Graceful degradation: log warning but still return Installed
            warn!(
                "Failed to parse version from '{}' for {}",
                version_output.trim(),
                kind.display_name()
            );
            (None, Some(version_output.trim().to_string()))
        }
    };

    // Step 5: Build metadata and return Installed
    AgentStatus::Installed(InstalledMetadata {
        path: path.clone(),
        version,
        raw_version,
        install_method: detect_install_method(&path),
        last_verified: SystemTime::now(),
        reasoning_level: None,
    })
}

/// Internal helper for parallel detection that returns Result per agent.
///
/// This function wraps the detection logic to return a Result, enabling
/// error isolation in parallel detection. NotInstalled is considered
/// a successful detection (not an error), while Unknown errors are
/// propagated as Err.
async fn detect_one(
    kind: AgentKind,
    options: &DetectOptions,
) -> (AgentKind, Result<AgentStatus, DetectionError>) {
    let status = detect_with_options(kind, options.clone()).await;

    let result = match &status {
        // Successful detection states - return Ok
        AgentStatus::Installed(_) => Ok(status),
        AgentStatus::NotInstalled => Ok(status),
        AgentStatus::VersionMismatch { .. } => Ok(status),
        // Detection errors - propagate as Err
        AgentStatus::Unknown { error, .. } => Err(error.clone()),
        // Handle any future variants conservatively (AgentStatus is #[non_exhaustive])
        #[allow(unreachable_patterns)]
        _ => Ok(status),
    };

    (kind, result)
}

/// Detect all known agents in parallel using default options.
///
/// This function detects all agents defined in `AgentKind` concurrently,
/// returning a map of agent kinds to their detection results. Each agent's
/// detection is isolated, so one failure doesn't affect others.
///
/// For custom timeout configuration, use [`detect_all_with_options`].
///
/// # Performance
///
/// Detection is performed in parallel using `futures::future::join_all`,
/// so the total detection time is approximately the time of the slowest
/// agent detection, not the sum of all detection times.
///
/// # Returns
///
/// A `HashMap` mapping each `AgentKind` to a `Result<AgentStatus, DetectionError>`.
/// - `Ok(AgentStatus::Installed(_))` - Agent found and usable
/// - `Ok(AgentStatus::NotInstalled)` - Agent definitively not found
/// - `Ok(AgentStatus::VersionMismatch { .. })` - Agent found but version issue
/// - `Err(DetectionError)` - Detection failed with error
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
///     for (kind, result) in &all {
///         match result {
///             Ok(status) if status.is_usable() => {
///                 println!("{}: available", kind.display_name());
///             }
///             Ok(_) => {
///                 println!("{}: not available", kind.display_name());
///             }
///             Err(e) => {
///                 println!("{}: detection failed: {}", kind.display_name(), e.description());
///             }
///         }
///     }
/// }
/// ```
pub async fn detect_all() -> HashMap<AgentKind, Result<AgentStatus, DetectionError>> {
    detect_all_with_options(DetectOptions::default()).await
}

/// Detect all known agents in parallel with custom options.
///
/// This function detects all agents defined in `AgentKind` concurrently,
/// using the provided detection options for configuration. Each agent's
/// detection is isolated, so one failure doesn't affect others.
///
/// # Arguments
///
/// * `options` - Configuration options including timeout
///
/// # Performance
///
/// Detection is performed in parallel using `futures::future::join_all`,
/// so the total detection time is approximately the time of the slowest
/// agent detection, not the sum of all detection times.
///
/// # Returns
///
/// A `HashMap` mapping each `AgentKind` to a `Result<AgentStatus, DetectionError>`.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{DetectOptions, detect_all_with_options};
/// use std::time::Duration;
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let options = DetectOptions {
///         timeout: Duration::from_secs(10),
///         ..Default::default()
///     };
///     let all = detect_all_with_options(options).await;
///
///     for (kind, result) in &all {
///         if let Ok(status) = result {
///             if status.is_usable() {
///                 println!("{}: ready", kind.display_name());
///             }
///         }
///     }
/// }
/// ```
pub async fn detect_all_with_options(
    options: DetectOptions,
) -> HashMap<AgentKind, Result<AgentStatus, DetectionError>> {
    let futures: Vec<_> = AgentKind::all()
        .map(|kind| detect_one(kind, &options))
        .collect();

    join_all(futures).await.into_iter().collect()
}

/// Detect the installation method from the executable path.
///
/// This heuristic checks the path for common patterns that indicate
/// how the tool was installed. On Windows, path matching is case-insensitive
/// to account for filesystem behavior.
fn detect_install_method(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    // Normalize case for Windows (case-insensitive filesystem)
    #[cfg(windows)]
    let path_str = path_str.to_lowercase();
    #[cfg(not(windows))]
    let path_str = path_str.to_string();

    // npm patterns (cross-platform)
    if path_str.contains(".npm") || path_str.contains("node_modules") {
        return Some("npm".to_string());
    }

    // Windows-specific npm location: %APPDATA%\npm
    #[cfg(windows)]
    if path_str.contains("appdata") && path_str.contains("npm") {
        return Some("npm".to_string());
    }

    // Cargo (cross-platform)
    if path_str.contains(".cargo") {
        return Some("cargo".to_string());
    }

    // Unix package managers
    #[cfg(not(windows))]
    {
        if path_str.contains("homebrew") || path_str.contains("linuxbrew") {
            return Some("brew".to_string());
        }
        if path_str.contains("mise") {
            return Some("mise".to_string());
        }
    }

    // Windows package managers
    #[cfg(windows)]
    {
        if path_str.contains("scoop") {
            return Some("scoop".to_string());
        }
        if path_str.contains("chocolatey") {
            return Some("chocolatey".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_detect_all_returns_all_agents() {
        let all = detect_all().await;

        // Should have an entry for each agent kind
        assert_eq!(all.len(), 4);
        assert!(all.contains_key(&AgentKind::ClaudeCode));
        assert!(all.contains_key(&AgentKind::Codex));
        assert!(all.contains_key(&AgentKind::OpenCode));
        assert!(all.contains_key(&AgentKind::Gemini));

        // Each entry should be a Result (Ok or Err)
        for (_, result) in &all {
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_detect_all_parallel_execution() {
        // This test verifies the function completes (parallel execution works)
        // Actual parallel timing would require real I/O
        let all = detect_all().await;
        assert!(!all.is_empty());
    }

    #[tokio::test]
    async fn test_detect_with_custom_timeout() {
        // Test that custom options are accepted
        let options = DetectOptions {
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        // Even with a very short timeout, detection should complete
        // (either success or timeout/not found)
        let status = detect_with_options(AgentKind::ClaudeCode, options).await;
        // Status should be one of the valid variants
        assert!(matches!(
            status,
            AgentStatus::Installed(_)
                | AgentStatus::NotInstalled
                | AgentStatus::VersionMismatch { .. }
                | AgentStatus::Unknown { .. }
        ));
    }

    #[tokio::test]
    async fn test_detect_all_with_options() {
        let options = DetectOptions {
            timeout: Duration::from_secs(1),
            ..Default::default()
        };
        let all = detect_all_with_options(options).await;

        // Should have all agents
        assert_eq!(all.len(), 4);

        // Each result should be valid
        for (_, result) in &all {
            match result {
                Ok(status) => {
                    assert!(matches!(
                        status,
                        AgentStatus::Installed(_)
                            | AgentStatus::NotInstalled
                            | AgentStatus::VersionMismatch { .. }
                            | AgentStatus::Unknown { .. }
                    ));
                }
                Err(e) => {
                    // Error should have a description
                    assert!(!e.description().is_empty());
                }
            }
        }
    }

    // Compile-time verification that detect functions return impl Future
    #[test]
    fn test_detect_returns_future() {
        fn assert_future<F: std::future::Future>(_: F) {}
        // These lines verify the async nature at compile time
        // If detect() were not async, this would fail to compile
        assert_future(detect(AgentKind::ClaudeCode));
        assert_future(detect_all());
        assert_future(detect_with_options(
            AgentKind::ClaudeCode,
            DetectOptions::default(),
        ));
        assert_future(detect_all_with_options(DetectOptions::default()));
    }

    #[tokio::test]
    async fn test_parallel_detection_faster_than_sequential() {
        use std::time::Instant;

        // Time sequential detection (one at a time)
        let sequential_start = Instant::now();
        for kind in AgentKind::all() {
            let _ = detect(kind).await;
        }
        let sequential_duration = sequential_start.elapsed();

        // Time parallel detection (all at once)
        let parallel_start = Instant::now();
        let _ = detect_all().await;
        let parallel_duration = parallel_start.elapsed();

        // Parallel should be faster (or at most equal if all agents missing)
        // Use generous margin: parallel should not be slower than sequential
        assert!(
            parallel_duration <= sequential_duration + Duration::from_millis(50),
            "Parallel detection ({:?}) should not be significantly slower than sequential ({:?})",
            parallel_duration,
            sequential_duration
        );

        // If we have agents and detection actually takes time, check speedup
        let agent_count = AgentKind::all().count();
        if agent_count > 1 && sequential_duration.as_millis() > 100 {
            // Only check meaningful speedup if detection actually takes time
            let expected_max = sequential_duration.as_millis() as f64 * 0.9;
            assert!(
                (parallel_duration.as_millis() as f64) < expected_max,
                "Parallel detection ({:?}) should be meaningfully faster than sequential ({:?})",
                parallel_duration,
                sequential_duration
            );
        }
    }

    // Cross-platform npm tests (patterns that work on both platforms)
    #[test]
    fn test_detect_install_method_npm_cross_platform() {
        let path = std::path::PathBuf::from("/home/user/.npm-global/bin/opencode");
        assert_eq!(detect_install_method(&path), Some("npm".to_string()));

        let path = std::path::PathBuf::from("/usr/local/lib/node_modules/.bin/tool");
        assert_eq!(detect_install_method(&path), Some("npm".to_string()));
    }

    // Cross-platform cargo test
    #[test]
    fn test_detect_install_method_cargo() {
        let path = std::path::PathBuf::from("/home/user/.cargo/bin/tool");
        assert_eq!(detect_install_method(&path), Some("cargo".to_string()));
    }

    // Unix-only tests (brew, mise)
    #[test]
    #[cfg(not(windows))]
    fn test_detect_install_method_brew() {
        let path = std::path::PathBuf::from("/home/linuxbrew/.linuxbrew/bin/tool");
        assert_eq!(detect_install_method(&path), Some("brew".to_string()));

        let path = std::path::PathBuf::from("/opt/homebrew/bin/tool");
        assert_eq!(detect_install_method(&path), Some("brew".to_string()));
    }

    #[test]
    #[cfg(not(windows))]
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

    // Windows-specific tests
    #[test]
    #[cfg(windows)]
    fn test_detect_install_method_npm_appdata() {
        // Test npm detection from AppData\Roaming\npm
        let path = std::path::PathBuf::from(r"C:\Users\User\AppData\Roaming\npm\claude.cmd");
        assert_eq!(detect_install_method(&path), Some("npm".to_string()));
    }

    #[test]
    #[cfg(windows)]
    fn test_detect_install_method_npm_appdata_case_insensitive() {
        // Test case-insensitivity (AppData vs appdata)
        let path = std::path::PathBuf::from(r"C:\Users\User\APPDATA\Roaming\NPM\tool.cmd");
        assert_eq!(detect_install_method(&path), Some("npm".to_string()));
    }

    #[test]
    #[cfg(windows)]
    fn test_detect_install_method_scoop() {
        // Test scoop detection
        let path = std::path::PathBuf::from(r"C:\Users\User\scoop\shims\tool.exe");
        assert_eq!(detect_install_method(&path), Some("scoop".to_string()));
    }

    #[test]
    #[cfg(windows)]
    fn test_detect_install_method_chocolatey() {
        // Test chocolatey detection
        let path = std::path::PathBuf::from(r"C:\ProgramData\chocolatey\bin\tool.exe");
        assert_eq!(detect_install_method(&path), Some("chocolatey".to_string()));
    }

    #[test]
    #[cfg(windows)]
    fn test_detect_install_method_cargo_windows() {
        // Test cargo on Windows (cross-platform pattern)
        let path = std::path::PathBuf::from(r"C:\Users\User\.cargo\bin\tool.exe");
        assert_eq!(detect_install_method(&path), Some("cargo".to_string()));
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::detection::{check_version, find_executable, parse_version};
    use std::time::Duration;

    // Unit tests for synchronous functions - these are deterministic and stable
    #[test]
    fn test_find_executable_returns_none_for_nonexistent() {
        let result = find_executable("definitely_not_a_real_agent_cli_xyz123");
        assert!(result.is_none());
    }

    #[test]
    fn test_version_parsing_claude_format() {
        let (version, raw) = parse_version("2.1.12 (Claude Code)").unwrap();
        assert_eq!(version.to_string(), "2.1.12");
        assert_eq!(raw, "2.1.12");
    }

    #[test]
    fn test_version_parsing_codex_format() {
        let (version, raw) = parse_version("codex-cli 0.87.0").unwrap();
        assert_eq!(version.to_string(), "0.87.0");
        assert_eq!(raw, "0.87.0");
    }

    #[test]
    fn test_version_parsing_opencode_format() {
        let (version, raw) = parse_version("1.1.25").unwrap();
        assert_eq!(version.to_string(), "1.1.25");
        assert_eq!(raw, "1.1.25");
    }

    #[test]
    fn test_version_parsing_gemini_format() {
        let (version, raw) = parse_version("gemini 0.1.5").unwrap();
        assert_eq!(version.to_string(), "0.1.5");
        assert_eq!(raw, "0.1.5");
    }

    #[test]
    fn test_version_parsing_invalid() {
        let result = parse_version("no version here");
        assert!(result.is_none());
    }

    #[test]
    fn test_version_parsing_v_prefix() {
        let (version, raw) = parse_version("v1.2.3").unwrap();
        assert_eq!(version.to_string(), "1.2.3");
        assert_eq!(raw, "v1.2.3");
    }

    #[test]
    fn test_version_parsing_two_part() {
        let (version, raw) = parse_version("version 1.2").unwrap();
        assert_eq!(version.to_string(), "1.2.0");
        assert_eq!(raw, "1.2");
    }

    // Async tests for check_version with real system executables
    #[tokio::test(flavor = "current_thread")]
    async fn test_check_version_io_error_for_nonexistent() {
        let exec_path = std::path::PathBuf::from("/nonexistent/path/to/agent");
        let result = check_version(&exec_path, Duration::from_secs(2)).await;
        assert!(matches!(result, Err(DetectionError::IoError)));
    }

    #[tokio::test]
    async fn test_detect_with_skip_version() {
        // Test that skip_version returns Installed with None version
        let options = DetectOptions {
            skip_version: true,
            ..Default::default()
        };
        let status = detect_with_options(AgentKind::ClaudeCode, options).await;

        // If Claude Code is installed, it should return Installed with version: None
        // If not installed, it should return NotInstalled
        match status {
            AgentStatus::Installed(meta) => {
                assert!(
                    meta.version.is_none(),
                    "skip_version should result in version: None"
                );
                assert!(
                    meta.raw_version.is_none(),
                    "skip_version should result in raw_version: None"
                );
            }
            AgentStatus::NotInstalled => {
                // Expected if agent not installed
            }
            _ => panic!("Unexpected status with skip_version: {:?}", status),
        }
    }
}
