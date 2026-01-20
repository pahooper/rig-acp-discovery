//! Detection options configuration.
//!
//! This module provides the [`DetectOptions`] struct for configuring
//! agent detection behavior, including timeouts and version parsing options.

use std::time::Duration;

/// Configuration options for agent detection.
///
/// This struct allows customization of the detection process,
/// such as setting a custom timeout for version checks or skipping
/// version parsing entirely for faster detection.
///
/// # Default Behavior
///
/// The default timeout is 5 seconds, which is suitable for most
/// systems. On slower systems or when detecting agents over network
/// mounts, you may want to increase this value.
///
/// By default, version parsing is enabled (`skip_version: false`).
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::DetectOptions;
/// use std::time::Duration;
///
/// // Use default options (5 second timeout, version parsing enabled)
/// let opts = DetectOptions::default();
///
/// // Use custom timeout
/// let opts = DetectOptions {
///     timeout: Duration::from_secs(10),
///     ..Default::default()
/// };
///
/// // Fast-path detection (skip version parsing)
/// let opts = DetectOptions {
///     skip_version: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct DetectOptions {
    /// Timeout for version check execution.
    ///
    /// This is the maximum time to wait for the agent's `--version`
    /// command to complete. If the command takes longer than this,
    /// the detection will return a timeout error.
    ///
    /// Default: 5 seconds
    pub timeout: Duration,

    /// Skip version parsing for fast-path detection.
    ///
    /// When set to `true`, detection will skip running `--version` and
    /// parsing the output. The resulting `InstalledMetadata` will have
    /// `version: None` and `raw_version: None`.
    ///
    /// This is useful when you only need to check if an agent exists,
    /// not what version it is. It can significantly speed up detection.
    ///
    /// Default: `false` (version parsing enabled)
    pub skip_version: bool,
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            skip_version: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeout() {
        let opts = DetectOptions::default();
        assert_eq!(opts.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_default_skip_version() {
        let opts = DetectOptions::default();
        assert!(!opts.skip_version);
    }

    #[test]
    fn test_custom_timeout() {
        let opts = DetectOptions {
            timeout: Duration::from_millis(500),
            ..Default::default()
        };
        assert_eq!(opts.timeout, Duration::from_millis(500));
        assert!(!opts.skip_version);
    }

    #[test]
    fn test_skip_version_option() {
        let opts = DetectOptions {
            skip_version: true,
            ..Default::default()
        };
        assert!(opts.skip_version);
        assert_eq!(opts.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_clone() {
        let opts = DetectOptions {
            timeout: Duration::from_secs(10),
            skip_version: true,
        };
        let cloned = opts.clone();
        assert_eq!(opts.timeout, cloned.timeout);
        assert_eq!(opts.skip_version, cloned.skip_version);
    }
}
