//! Installation execution for AI coding agents.
//!
//! This module provides the main [`install`] function that executes agent
//! installation with progress reporting, timeout handling, and verification.

use crate::install::{InstallError, InstallOptions, InstallProgress};
use crate::{detect, AgentKind};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::timeout;

/// Install an agent programmatically.
///
/// This function:
/// 1. Runs pre-flight checks (can_install)
/// 2. Reports progress via callback
/// 3. Executes the installer command with timeout
/// 4. Verifies installation via detect()
///
/// # Arguments
///
/// - `kind`: The agent to install
/// - `options`: Installation options (timeout, etc.)
/// - `on_progress`: Required callback for progress updates
///
/// # Returns
///
/// - `Ok(())` if installation and verification succeeded
/// - `Err(InstallError)` with actionable fix suggestion if failed
///
/// # Consent Model
///
/// Calling this function IS consent to install. The caller's UI
/// is responsible for confirming with the user before calling.
///
/// # Example
///
/// ```rust,no_run
/// use rig_acp_discovery::{AgentKind, InstallOptions, InstallProgress, install};
///
/// #[tokio::main]
/// async fn main() {
///     let result = install(
///         AgentKind::ClaudeCode,
///         InstallOptions::default(),
///         |progress| println!("{:?}", progress),
///     ).await;
///
///     match result {
///         Ok(()) => println!("Installed successfully!"),
///         Err(e) => println!("Failed: {}. Fix: {}", e, e.fix_suggestion()),
///     }
/// }
/// ```
pub async fn install<F>(kind: AgentKind, options: InstallOptions, on_progress: F) -> Result<(), InstallError>
where
    F: Fn(InstallProgress) + Send + Sync,
{
    // Step 1: Report Started
    on_progress(InstallProgress::Started { agent: kind });

    // Step 2: Pre-flight check
    on_progress(InstallProgress::CheckingPrerequisites);
    super::prereq::can_install(kind).await?;

    // Step 3: Get install info and build command
    let info = kind.install_info();
    let cmd = &info.primary.command;

    let mut command = Command::new(&cmd.program);
    command
        .args(&cmd.args)
        .envs(cmd.env_vars.iter().cloned())
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Step 4: Report Installing and execute with timeout
    on_progress(InstallProgress::Installing { agent: kind });

    let result = timeout(options.timeout, command.output()).await;

    // Step 5: Handle timeout and execution result
    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            // Check for permission denied
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(InstallError::PermissionDenied {
                    message: e.to_string(),
                    fix: "Try running with appropriate permissions".to_string(),
                });
            }
            return Err(InstallError::InstallerFailed {
                message: e.to_string(),
                exit_code: None,
                stdout: None,
                stderr: None,
                fix: "Check the command and try again".to_string(),
            });
        }
        Err(_) => {
            return Err(InstallError::Timeout {
                duration: options.timeout,
                fix: format!(
                    "Installation timed out after {:?}. Try with a longer timeout or check network.",
                    options.timeout
                ),
            });
        }
    };

    // Step 6: Check exit status
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Detect network errors from stderr
        let is_network = stderr.contains("network")
            || stderr.contains("connection")
            || stderr.contains("resolve")
            || stderr.contains("ETIMEDOUT")
            || stderr.contains("ENOTFOUND");

        if is_network {
            return Err(InstallError::Network {
                message: "Network error during installation".to_string(),
                stderr: Some(stderr),
                fix: "Check your internet connection and try again".to_string(),
            });
        }

        return Err(InstallError::InstallerFailed {
            message: format!("Installer exited with code {:?}", output.status.code()),
            exit_code: output.status.code(),
            stdout: Some(stdout),
            stderr: Some(stderr),
            fix: "See installer output above for details".to_string(),
        });
    }

    // Step 7: Verify installation
    on_progress(InstallProgress::Verifying { agent: kind });

    // Small delay for PATH to potentially update
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let status = detect(kind).await;
    if !status.is_usable() {
        return Err(InstallError::VerificationFailed {
            agent: kind,
            fix: "Installation completed but agent not found. You may need to restart your terminal for PATH changes to take effect.".to_string(),
        });
    }

    // Step 8: Report Completed
    on_progress(InstallProgress::Completed { agent: kind });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_install_progress_callback() {
        // Verify callback is called with expected progress stages
        let stages = Arc::new(Mutex::new(Vec::new()));
        let stages_clone = stages.clone();

        // Run install - it will fail at some point but should call callback
        let _ = install(AgentKind::ClaudeCode, InstallOptions::default(), move |progress| {
            stages_clone.lock().unwrap().push(format!("{:?}", progress));
        })
        .await;

        let stages = stages.lock().unwrap();
        // At minimum, Started should have been called
        assert!(!stages.is_empty(), "Progress callback should be called");
        assert!(stages[0].contains("Started"), "First stage should be Started");
    }

    #[tokio::test]
    async fn test_install_options_timeout() {
        let opts = InstallOptions {
            timeout: std::time::Duration::from_secs(1),
        };
        assert_eq!(opts.timeout.as_secs(), 1);
    }

    #[tokio::test]
    async fn test_install_prerequisite_check_runs() {
        // Verify that can_install is called (CheckingPrerequisites stage)
        let saw_prereq_check = Arc::new(Mutex::new(false));
        let saw_prereq_check_clone = saw_prereq_check.clone();

        let _ = install(AgentKind::ClaudeCode, InstallOptions::default(), move |progress| {
            if matches!(progress, InstallProgress::CheckingPrerequisites) {
                *saw_prereq_check_clone.lock().unwrap() = true;
            }
        })
        .await;

        assert!(
            *saw_prereq_check.lock().unwrap(),
            "Should see CheckingPrerequisites stage"
        );
    }

    #[tokio::test]
    async fn test_install_stages_order() {
        // Verify progress stages are emitted in correct order
        let stages = Arc::new(Mutex::new(Vec::new()));
        let stages_clone = stages.clone();

        let _ = install(AgentKind::ClaudeCode, InstallOptions::default(), move |progress| {
            let stage_name = match &progress {
                InstallProgress::Started { .. } => "Started",
                InstallProgress::CheckingPrerequisites => "CheckingPrerequisites",
                InstallProgress::Downloading { .. } => "Downloading",
                InstallProgress::Installing { .. } => "Installing",
                InstallProgress::Verifying { .. } => "Verifying",
                InstallProgress::Completed { .. } => "Completed",
            };
            stages_clone.lock().unwrap().push(stage_name.to_string());
        })
        .await;

        let stages = stages.lock().unwrap();
        // Verify order: Started -> CheckingPrerequisites -> (maybe more)
        assert!(stages.len() >= 2, "Should have at least 2 stages");
        assert_eq!(stages[0], "Started");
        assert_eq!(stages[1], "CheckingPrerequisites");
    }

    #[tokio::test]
    async fn test_install_with_short_timeout() {
        // Test that timeout error is returned with very short timeout
        let stages = Arc::new(Mutex::new(Vec::new()));
        let stages_clone = stages.clone();

        let result = install(
            AgentKind::ClaudeCode,
            InstallOptions {
                timeout: std::time::Duration::from_millis(1),
            },
            move |progress| {
                stages_clone.lock().unwrap().push(format!("{:?}", progress));
            },
        )
        .await;

        // Either timeout error or other error (depending on system state)
        assert!(result.is_err());

        // Callback should still have been called
        let stages = stages.lock().unwrap();
        assert!(!stages.is_empty());
    }
}
