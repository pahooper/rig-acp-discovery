//! Progress reporting types for installation operations.
//!
//! This module provides types for tracking and reporting installation progress.
//! The [`InstallProgress`] enum represents discrete stages of installation that
//! can be reported to users via a callback.

use crate::AgentKind;
use std::time::Duration;

/// Progress stages during agent installation.
///
/// Each variant represents a discrete stage of the installation process.
/// These can be used to provide user feedback (progress bars, status messages, etc.)
/// by passing a callback to the install function.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{AgentKind, InstallProgress};
///
/// fn on_progress(progress: InstallProgress) {
///     match &progress {
///         InstallProgress::Started { agent } => {
///             println!("Starting installation of {}", agent.display_name());
///         }
///         InstallProgress::CheckingPrerequisites => {
///             println!("Checking prerequisites...");
///         }
///         InstallProgress::Downloading { agent, estimated_remaining } => {
///             if let Some(remaining) = estimated_remaining {
///                 println!("Downloading {} ({:?} remaining)", agent.display_name(), remaining);
///             } else {
///                 println!("Downloading {}...", agent.display_name());
///             }
///         }
///         InstallProgress::Installing { agent } => {
///             println!("Installing {}...", agent.display_name());
///         }
///         InstallProgress::Verifying { agent } => {
///             println!("Verifying {} installation...", agent.display_name());
///         }
///         InstallProgress::Completed { agent } => {
///             println!("{} installed successfully!", agent.display_name());
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum InstallProgress {
    /// Installation has started.
    Started {
        /// The agent being installed.
        agent: AgentKind,
    },

    /// Checking prerequisites before installation.
    CheckingPrerequisites,

    /// Downloading the agent.
    Downloading {
        /// The agent being downloaded.
        agent: AgentKind,
        /// Estimated time remaining, if known.
        estimated_remaining: Option<Duration>,
    },

    /// Installing the agent.
    Installing {
        /// The agent being installed.
        agent: AgentKind,
    },

    /// Verifying the installation.
    Verifying {
        /// The agent being verified.
        agent: AgentKind,
    },

    /// Installation completed successfully.
    Completed {
        /// The agent that was installed.
        agent: AgentKind,
    },
}

impl InstallProgress {
    /// Get a human-readable description of the current progress stage.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::{AgentKind, InstallProgress};
    ///
    /// let progress = InstallProgress::CheckingPrerequisites;
    /// assert_eq!(progress.description(), "Checking prerequisites");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            Self::Started { .. } => "Starting installation",
            Self::CheckingPrerequisites => "Checking prerequisites",
            Self::Downloading { .. } => "Downloading",
            Self::Installing { .. } => "Installing",
            Self::Verifying { .. } => "Verifying installation",
            Self::Completed { .. } => "Installation complete",
        }
    }

    /// Check if this progress stage indicates completion.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::{AgentKind, InstallProgress};
    ///
    /// let progress = InstallProgress::Completed { agent: AgentKind::ClaudeCode };
    /// assert!(progress.is_complete());
    ///
    /// let progress = InstallProgress::Installing { agent: AgentKind::ClaudeCode };
    /// assert!(!progress.is_complete());
    /// ```
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }
}

/// Options for controlling installation behavior.
///
/// This struct allows customizing installation parameters such as timeout.
/// Use [`Default::default()`] for sensible defaults.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::InstallOptions;
/// use std::time::Duration;
///
/// // Use defaults (5 minute timeout)
/// let options = InstallOptions::default();
/// assert_eq!(options.timeout, Duration::from_secs(300));
///
/// // Custom timeout
/// let options = InstallOptions {
///     timeout: Duration::from_secs(600),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct InstallOptions {
    /// Maximum time to wait for installation to complete.
    ///
    /// Default: 5 minutes (300 seconds).
    pub timeout: Duration,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_progress_description() {
        assert_eq!(
            InstallProgress::Started {
                agent: AgentKind::ClaudeCode
            }
            .description(),
            "Starting installation"
        );
        assert_eq!(
            InstallProgress::CheckingPrerequisites.description(),
            "Checking prerequisites"
        );
        assert_eq!(
            InstallProgress::Downloading {
                agent: AgentKind::Codex,
                estimated_remaining: None
            }
            .description(),
            "Downloading"
        );
        assert_eq!(
            InstallProgress::Installing {
                agent: AgentKind::OpenCode
            }
            .description(),
            "Installing"
        );
        assert_eq!(
            InstallProgress::Verifying {
                agent: AgentKind::Gemini
            }
            .description(),
            "Verifying installation"
        );
        assert_eq!(
            InstallProgress::Completed {
                agent: AgentKind::ClaudeCode
            }
            .description(),
            "Installation complete"
        );
    }

    #[test]
    fn test_install_progress_is_complete() {
        assert!(InstallProgress::Completed {
            agent: AgentKind::ClaudeCode
        }
        .is_complete());

        assert!(!InstallProgress::Started {
            agent: AgentKind::ClaudeCode
        }
        .is_complete());
        assert!(!InstallProgress::CheckingPrerequisites.is_complete());
        assert!(!InstallProgress::Downloading {
            agent: AgentKind::Codex,
            estimated_remaining: Some(Duration::from_secs(30))
        }
        .is_complete());
        assert!(!InstallProgress::Installing {
            agent: AgentKind::OpenCode
        }
        .is_complete());
        assert!(!InstallProgress::Verifying {
            agent: AgentKind::Gemini
        }
        .is_complete());
    }

    #[test]
    fn test_install_options_default() {
        let opts = InstallOptions::default();
        assert_eq!(opts.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_install_options_custom() {
        let opts = InstallOptions {
            timeout: Duration::from_secs(600),
        };
        assert_eq!(opts.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_install_progress_clone() {
        let progress = InstallProgress::Downloading {
            agent: AgentKind::ClaudeCode,
            estimated_remaining: Some(Duration::from_secs(30)),
        };
        let cloned = progress.clone();
        assert_eq!(progress.description(), cloned.description());
    }

    #[test]
    fn test_install_options_clone() {
        let opts = InstallOptions {
            timeout: Duration::from_secs(120),
        };
        let cloned = opts.clone();
        assert_eq!(opts.timeout, cloned.timeout);
    }
}
