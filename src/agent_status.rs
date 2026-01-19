//! Agent status types representing detection results.

use semver::Version;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Metadata for an installed agent.
///
/// This struct contains detailed information about a successfully detected
/// and installed agent, including its location, version, and capabilities.
#[derive(Debug, Clone)]
pub struct InstalledMetadata {
    /// Path to the executable.
    pub path: PathBuf,

    /// Parsed semantic version of the agent.
    pub version: Version,

    /// How the agent was installed (e.g., "npm", "cargo", "manual").
    ///
    /// This is `None` if the installation method couldn't be determined.
    pub install_method: Option<String>,

    /// When detection was last verified.
    ///
    /// This timestamp indicates when the detection result was obtained,
    /// which can be used for cache invalidation.
    pub last_verified: SystemTime,

    /// Agent's reasoning level capability (raw string from agent).
    ///
    /// Different agents name their reasoning levels differently, so this
    /// stores the raw string from the agent. `None` indicates the agent
    /// doesn't support reasoning levels.
    pub reasoning_level: Option<String>,
}

/// Typed error variants for detection failures.
///
/// This enum categorizes the different ways detection can fail, allowing
/// callers to handle specific error types appropriately.
///
/// This enum is marked `#[non_exhaustive]` to allow adding new error types
/// in future versions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DetectionError {
    /// Timed out while detecting the agent.
    Timeout,

    /// Permission denied accessing the executable or its location.
    PermissionDenied,

    /// Failed to parse the version output from the agent.
    VersionParseFailed,

    /// I/O error during detection (e.g., failed to execute command).
    IoError,
}

impl DetectionError {
    /// Human-readable description of the error.
    ///
    /// This provides a user-friendly message suitable for display.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::DetectionError;
    ///
    /// let error = DetectionError::Timeout;
    /// assert_eq!(error.description(), "Detection timed out");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            Self::Timeout => "Detection timed out",
            Self::PermissionDenied => "Permission denied",
            Self::VersionParseFailed => "Failed to parse version",
            Self::IoError => "I/O error during detection",
        }
    }
}

/// Result of agent detection.
///
/// This enum represents the possible outcomes of detecting an AI coding agent.
/// Each variant provides the appropriate level of detail for that outcome.
///
/// # Variants
///
/// - `Installed`: Agent found and usable with full metadata
/// - `NotInstalled`: Agent definitively not found
/// - `VersionMismatch`: Agent found but version doesn't meet requirements
/// - `Unknown`: Detection failed with an error
///
/// # Extensibility
///
/// This enum is marked `#[non_exhaustive]` to allow adding new status types
/// in future versions.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::AgentStatus;
///
/// fn handle_status(status: AgentStatus) {
///     if status.is_usable() {
///         println!("Agent ready at {:?}", status.path());
///     } else if status.is_installed() {
///         println!("Agent found but version mismatch");
///     } else {
///         println!("Agent not available");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AgentStatus {
    /// Agent is installed and usable.
    Installed(InstalledMetadata),

    /// Agent is definitively not installed.
    NotInstalled,

    /// Agent found but version doesn't match requirements.
    VersionMismatch {
        /// The version that was found.
        found: Version,
        /// The required minimum version.
        required: Version,
        /// Path where the agent was found.
        path: PathBuf,
    },

    /// Detection failed with an error.
    Unknown {
        /// Typed error variant for programmatic handling.
        error: DetectionError,
        /// Human-readable message for display.
        message: String,
    },
}

impl AgentStatus {
    /// Check if the agent is usable (installed and correct version).
    ///
    /// Returns `true` only for the `Installed` variant, meaning the agent
    /// is ready to use without any additional action.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentStatus;
    ///
    /// let status = AgentStatus::NotInstalled;
    /// assert!(!status.is_usable());
    /// ```
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Installed(_))
    }

    /// Check if the agent is installed (regardless of version).
    ///
    /// Returns `true` for both `Installed` and `VersionMismatch` variants,
    /// indicating that the agent binary exists on the system.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentStatus;
    ///
    /// let status = AgentStatus::NotInstalled;
    /// assert!(!status.is_installed());
    /// ```
    pub fn is_installed(&self) -> bool {
        matches!(self, Self::Installed(_) | Self::VersionMismatch { .. })
    }

    /// Get the path to the agent executable if available.
    ///
    /// Returns `Some(&Path)` for `Installed` and `VersionMismatch` variants,
    /// `None` for `NotInstalled` and `Unknown`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentStatus;
    ///
    /// let status = AgentStatus::NotInstalled;
    /// assert!(status.path().is_none());
    /// ```
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Installed(meta) => Some(&meta.path),
            Self::VersionMismatch { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Get the version of the agent if available.
    ///
    /// Returns `Some(&Version)` for `Installed` and `VersionMismatch` variants,
    /// `None` for `NotInstalled` and `Unknown`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentStatus;
    ///
    /// let status = AgentStatus::NotInstalled;
    /// assert!(status.version().is_none());
    /// ```
    pub fn version(&self) -> Option<&Version> {
        match self {
            Self::Installed(meta) => Some(&meta.version),
            Self::VersionMismatch { found, .. } => Some(found),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_installed_metadata() -> InstalledMetadata {
        InstalledMetadata {
            path: PathBuf::from("/usr/bin/claude"),
            version: Version::parse("1.2.3").unwrap(),
            install_method: Some("npm".to_string()),
            last_verified: SystemTime::now(),
            reasoning_level: Some("high".to_string()),
        }
    }

    #[test]
    fn test_installed_status() {
        let meta = make_installed_metadata();
        let status = AgentStatus::Installed(meta);

        assert!(status.is_usable());
        assert!(status.is_installed());
        assert_eq!(status.path(), Some(Path::new("/usr/bin/claude")));
        assert_eq!(status.version(), Some(&Version::parse("1.2.3").unwrap()));
    }

    #[test]
    fn test_not_installed_status() {
        let status = AgentStatus::NotInstalled;

        assert!(!status.is_usable());
        assert!(!status.is_installed());
        assert!(status.path().is_none());
        assert!(status.version().is_none());
    }

    #[test]
    fn test_version_mismatch_status() {
        let status = AgentStatus::VersionMismatch {
            found: Version::parse("1.0.0").unwrap(),
            required: Version::parse("2.0.0").unwrap(),
            path: PathBuf::from("/usr/bin/claude"),
        };

        assert!(!status.is_usable());
        assert!(status.is_installed());
        assert_eq!(status.path(), Some(Path::new("/usr/bin/claude")));
        assert_eq!(status.version(), Some(&Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn test_unknown_status() {
        let status = AgentStatus::Unknown {
            error: DetectionError::Timeout,
            message: "Timed out after 5s".to_string(),
        };

        assert!(!status.is_usable());
        assert!(!status.is_installed());
        assert!(status.path().is_none());
        assert!(status.version().is_none());
    }

    #[test]
    fn test_detection_error_descriptions() {
        assert_eq!(DetectionError::Timeout.description(), "Detection timed out");
        assert_eq!(
            DetectionError::PermissionDenied.description(),
            "Permission denied"
        );
        assert_eq!(
            DetectionError::VersionParseFailed.description(),
            "Failed to parse version"
        );
        assert_eq!(
            DetectionError::IoError.description(),
            "I/O error during detection"
        );
    }

    #[test]
    fn test_detection_error_equality() {
        assert_eq!(DetectionError::Timeout, DetectionError::Timeout);
        assert_ne!(DetectionError::Timeout, DetectionError::IoError);
    }

    #[test]
    fn test_installed_metadata_clone() {
        let meta = make_installed_metadata();
        let cloned = meta.clone();

        assert_eq!(meta.path, cloned.path);
        assert_eq!(meta.version, cloned.version);
        assert_eq!(meta.install_method, cloned.install_method);
        assert_eq!(meta.reasoning_level, cloned.reasoning_level);
    }
}
