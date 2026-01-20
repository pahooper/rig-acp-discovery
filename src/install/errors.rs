//! Error types for installation operations.
//!
//! This module defines the error types that can occur during agent installation.
//! Each error variant includes an actionable fix suggestion to help users
//! resolve the issue.

use crate::AgentKind;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during agent installation.
///
/// Each variant includes contextual information about what went wrong and
/// a `fix` field with an actionable suggestion for resolving the issue.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::InstallError;
///
/// fn handle_error(error: InstallError) {
///     eprintln!("Installation failed: {}", error);
///     eprintln!("To fix: {}", error.fix_suggestion());
/// }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InstallError {
    /// A required prerequisite is missing.
    ///
    /// For example, Codex requires Node.js 18+ to be installed.
    #[error("Missing prerequisite: {name}")]
    PrerequisiteMissing {
        /// Name of the missing prerequisite (e.g., "Node.js 18+").
        name: String,
        /// URL where the prerequisite can be downloaded.
        install_url: Option<String>,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// A prerequisite is installed but doesn't meet version requirements.
    #[error("Prerequisite version mismatch: {name} requires {required}, found {found}")]
    PrerequisiteVersionMismatch {
        /// Name of the prerequisite.
        name: String,
        /// Required version (e.g., "18+").
        required: String,
        /// Version that was found.
        found: String,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// A network error occurred during installation.
    ///
    /// This typically indicates connectivity issues or problems downloading
    /// the agent from its source.
    #[error("Network error: {message}")]
    Network {
        /// Description of the network error.
        message: String,
        /// Standard error output from the failed command, if available.
        stderr: Option<String>,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// Permission was denied during installation.
    ///
    /// This can occur when trying to install to a location that requires
    /// elevated privileges without having them.
    #[error("Permission denied: {message}")]
    PermissionDenied {
        /// Description of what permission was denied.
        message: String,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// Installation timed out.
    ///
    /// The installation process did not complete within the configured timeout.
    #[error("Installation timed out after {duration:?}")]
    Timeout {
        /// How long the installation was allowed to run.
        duration: Duration,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// The installer process failed.
    ///
    /// This is the most common error type, indicating that the installation
    /// command returned an error.
    #[error("Installation failed: {message}")]
    InstallerFailed {
        /// Description of the failure.
        message: String,
        /// Exit code from the installer, if available.
        exit_code: Option<i32>,
        /// Standard output from the installer, if available.
        stdout: Option<String>,
        /// Standard error from the installer, if available.
        stderr: Option<String>,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// Installation completed but verification failed.
    ///
    /// The installer ran successfully, but the agent could not be detected
    /// afterward. This may indicate a PATH issue or incomplete installation.
    #[error("Verification failed: agent not detected after installation")]
    VerificationFailed {
        /// The agent that was being installed.
        agent: AgentKind,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },

    /// The agent is not supported on this platform.
    ///
    /// Some agents may not be available on certain operating systems.
    #[error("Platform not supported for {agent:?}")]
    UnsupportedPlatform {
        /// The agent that is not supported.
        agent: AgentKind,
        /// Actionable suggestion for resolving the issue.
        fix: String,
    },
}

impl InstallError {
    /// Get an actionable suggestion for fixing this error.
    ///
    /// Every error variant includes a fix suggestion that users can follow
    /// to resolve the issue.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::InstallError;
    /// use std::time::Duration;
    ///
    /// let error = InstallError::Timeout {
    ///     duration: Duration::from_secs(300),
    ///     fix: "Try again with a longer timeout or check network connectivity".to_string(),
    /// };
    /// assert!(error.fix_suggestion().contains("timeout"));
    /// ```
    pub fn fix_suggestion(&self) -> &str {
        match self {
            Self::PrerequisiteMissing { fix, .. } => fix,
            Self::PrerequisiteVersionMismatch { fix, .. } => fix,
            Self::Network { fix, .. } => fix,
            Self::PermissionDenied { fix, .. } => fix,
            Self::Timeout { fix, .. } => fix,
            Self::InstallerFailed { fix, .. } => fix,
            Self::VerificationFailed { fix, .. } => fix,
            Self::UnsupportedPlatform { fix, .. } => fix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_error_display() {
        let error = InstallError::PrerequisiteMissing {
            name: "Node.js".to_string(),
            install_url: Some("https://nodejs.org".to_string()),
            fix: "Install Node.js from https://nodejs.org".to_string(),
        };
        assert!(error.to_string().contains("Node.js"));
        assert!(error.to_string().contains("Missing prerequisite"));
    }

    #[test]
    fn test_fix_suggestion() {
        let error = InstallError::Timeout {
            duration: Duration::from_secs(300),
            fix: "Try again with a longer timeout".to_string(),
        };
        assert_eq!(error.fix_suggestion(), "Try again with a longer timeout");
    }

    #[test]
    fn test_all_variants_have_fix() {
        // Test that fix_suggestion() works and returns non-empty for each variant
        let errors = vec![
            InstallError::PrerequisiteMissing {
                name: "Node.js".to_string(),
                install_url: Some("https://nodejs.org".to_string()),
                fix: "Install Node.js".to_string(),
            },
            InstallError::PrerequisiteVersionMismatch {
                name: "Node.js".to_string(),
                required: "18+".to_string(),
                found: "16.0.0".to_string(),
                fix: "Upgrade Node.js to 18+".to_string(),
            },
            InstallError::Network {
                message: "Connection refused".to_string(),
                stderr: None,
                fix: "Check network connectivity".to_string(),
            },
            InstallError::PermissionDenied {
                message: "Cannot write to /usr/local/bin".to_string(),
                fix: "Use --user flag or run with sudo".to_string(),
            },
            InstallError::Timeout {
                duration: Duration::from_secs(300),
                fix: "Try again with longer timeout".to_string(),
            },
            InstallError::InstallerFailed {
                message: "npm install failed".to_string(),
                exit_code: Some(1),
                stdout: None,
                stderr: Some("EACCES".to_string()),
                fix: "Check npm permissions".to_string(),
            },
            InstallError::VerificationFailed {
                agent: AgentKind::ClaudeCode,
                fix: "Check PATH and restart terminal".to_string(),
            },
            InstallError::UnsupportedPlatform {
                agent: AgentKind::Codex,
                fix: "Use WSL on Windows".to_string(),
            },
        ];

        for error in errors {
            let fix = error.fix_suggestion();
            assert!(
                !fix.is_empty(),
                "fix_suggestion() should return non-empty string for {:?}",
                error
            );
        }
    }

    #[test]
    fn test_prerequisite_missing_display() {
        let error = InstallError::PrerequisiteMissing {
            name: "Node.js 18+".to_string(),
            install_url: Some("https://nodejs.org".to_string()),
            fix: "Install Node.js 18+ from https://nodejs.org".to_string(),
        };
        assert_eq!(error.to_string(), "Missing prerequisite: Node.js 18+");
    }

    #[test]
    fn test_version_mismatch_display() {
        let error = InstallError::PrerequisiteVersionMismatch {
            name: "Node.js".to_string(),
            required: "18+".to_string(),
            found: "16.0.0".to_string(),
            fix: "Upgrade Node.js".to_string(),
        };
        assert!(error.to_string().contains("Node.js"));
        assert!(error.to_string().contains("18+"));
        assert!(error.to_string().contains("16.0.0"));
    }

    #[test]
    fn test_verification_failed_display() {
        let error = InstallError::VerificationFailed {
            agent: AgentKind::ClaudeCode,
            fix: "Check PATH".to_string(),
        };
        assert!(error.to_string().contains("Verification failed"));
    }

    #[test]
    fn test_unsupported_platform_display() {
        let error = InstallError::UnsupportedPlatform {
            agent: AgentKind::Codex,
            fix: "Use WSL".to_string(),
        };
        assert!(error.to_string().contains("Platform not supported"));
    }
}
