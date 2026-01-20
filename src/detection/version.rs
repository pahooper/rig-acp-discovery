//! Async version check with timeout.

use crate::DetectionError;
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Check the version of an executable.
///
/// This function runs the executable with `--version` and captures its output.
/// The execution is wrapped in a configurable timeout to avoid hanging on
/// unresponsive or stuck processes. The spawned process is killed on drop
/// to prevent orphan processes when the future is cancelled.
///
/// # Arguments
///
/// * `path` - Path to the executable to check
/// * `timeout_duration` - Maximum time to wait for the command to complete
///
/// # Returns
///
/// `Ok(String)` with the version output (stdout preferred, stderr fallback),
/// or a `DetectionError` on failure:
/// - `Timeout` if the command takes longer than the specified timeout
/// - `PermissionDenied` if the executable cannot be run due to permissions
/// - `IoError` for other I/O failures or non-zero exit codes
/// - `VersionParseFailed` if output is not valid UTF-8
pub(crate) async fn check_version(
    path: &Path,
    timeout_duration: Duration,
) -> Result<String, DetectionError> {
    let mut cmd = Command::new(path);
    cmd.arg("--version").kill_on_drop(true);

    let output = timeout(timeout_duration, cmd.output())
    .await
    .map_err(|_| DetectionError::Timeout)?
    .map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            DetectionError::PermissionDenied
        } else {
            DetectionError::IoError
        }
    })?;

    if !output.status.success() {
        return Err(DetectionError::IoError);
    }

    // Try stdout first, fall back to stderr (some tools write version to stderr)
    let out = if !output.stdout.is_empty() {
        output.stdout
    } else {
        output.stderr
    };

    String::from_utf8(out).map_err(|_| DetectionError::VersionParseFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Default timeout for tests.
    const TEST_TIMEOUT: Duration = Duration::from_secs(2);

    #[tokio::test]
    async fn test_check_version_common_tool() {
        // ls --version should work on Linux
        let path = PathBuf::from("/bin/ls");
        if path.exists() {
            let result = check_version(&path, TEST_TIMEOUT).await;
            // Should succeed or fail gracefully (ls --version behavior varies)
            // On some systems ls might not have --version
            assert!(result.is_ok() || matches!(result, Err(DetectionError::IoError)));
        }
    }

    #[tokio::test]
    async fn test_check_version_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/to/executable");
        let result = check_version(&path, TEST_TIMEOUT).await;
        assert!(matches!(result, Err(DetectionError::IoError)));
    }

    #[tokio::test]
    async fn test_check_version_with_custom_timeout() {
        // Test that a very short timeout still works (though may timeout)
        let path = PathBuf::from("/nonexistent/path/to/executable");
        let result = check_version(&path, Duration::from_millis(100)).await;
        // Should fail with IoError (not timeout, since executable doesn't exist)
        assert!(matches!(result, Err(DetectionError::IoError)));
    }
}
