//! Async version check with timeout.

use crate::DetectionError;
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Timeout for version check execution.
const VERSION_TIMEOUT: Duration = Duration::from_secs(2);

/// Check the version of an executable.
///
/// This function runs the executable with `--version` and captures its output.
/// The execution is wrapped in a 2-second timeout to avoid hanging on
/// unresponsive or stuck processes.
///
/// # Arguments
///
/// * `path` - Path to the executable to check
///
/// # Returns
///
/// `Ok(String)` with the version output (stdout preferred, stderr fallback),
/// or a `DetectionError` on failure:
/// - `Timeout` if the command takes longer than 2 seconds
/// - `PermissionDenied` if the executable cannot be run due to permissions
/// - `IoError` for other I/O failures or non-zero exit codes
/// - `VersionParseFailed` if output is not valid UTF-8
pub(crate) async fn check_version(path: &Path) -> Result<String, DetectionError> {
    let output = timeout(
        VERSION_TIMEOUT,
        Command::new(path).arg("--version").output(),
    )
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

    #[tokio::test]
    async fn test_check_version_common_tool() {
        // ls --version should work on Linux
        let path = PathBuf::from("/bin/ls");
        if path.exists() {
            let result = check_version(&path).await;
            // Should succeed or fail gracefully (ls --version behavior varies)
            // On some systems ls might not have --version
            assert!(result.is_ok() || matches!(result, Err(DetectionError::IoError)));
        }
    }

    #[tokio::test]
    async fn test_check_version_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/to/executable");
        let result = check_version(&path).await;
        assert!(matches!(result, Err(DetectionError::IoError)));
    }
}
