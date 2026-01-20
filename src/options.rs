//! Detection options configuration.
//!
//! This module provides the [`DetectOptions`] struct for configuring
//! agent detection behavior, including timeouts.

use std::time::Duration;

/// Configuration options for agent detection.
///
/// This struct allows customization of the detection process,
/// such as setting a custom timeout for version checks.
///
/// # Default Behavior
///
/// The default timeout is 5 seconds, which is suitable for most
/// systems. On slower systems or when detecting agents over network
/// mounts, you may want to increase this value.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::DetectOptions;
/// use std::time::Duration;
///
/// // Use default options (5 second timeout)
/// let opts = DetectOptions::default();
///
/// // Use custom timeout
/// let opts = DetectOptions {
///     timeout: Duration::from_secs(10),
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
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
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
    fn test_custom_timeout() {
        let opts = DetectOptions {
            timeout: Duration::from_millis(500),
        };
        assert_eq!(opts.timeout, Duration::from_millis(500));
    }

    #[test]
    fn test_clone() {
        let opts = DetectOptions {
            timeout: Duration::from_secs(10),
        };
        let cloned = opts.clone();
        assert_eq!(opts.timeout, cloned.timeout);
    }
}
