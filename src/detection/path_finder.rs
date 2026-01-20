//! PATH-based executable lookup with fallback locations.

use std::path::PathBuf;

/// System fallback paths to check if executable not found in PATH.
const FALLBACK_PATHS: &[&str] = &["/usr/local/bin", "/usr/bin"];

/// Find an executable by name.
///
/// This function first tries to find the executable using the system PATH
/// via the `which` crate. If not found, it checks common fallback locations
/// including system directories and user home directories.
///
/// # Arguments
///
/// * `name` - The executable name to search for (e.g., "claude", "codex")
///
/// # Returns
///
/// `Some(PathBuf)` if the executable is found, `None` otherwise.
pub(crate) fn find_executable(name: &str) -> Option<PathBuf> {
    // Primary: PATH lookup via which crate
    // This handles symlinks, relative paths, and platform differences
    if let Ok(path) = which::which(name) {
        return Some(path);
    }

    // Fallback: common system locations not always in PATH
    for dir in FALLBACK_PATHS {
        let path = PathBuf::from(dir).join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Home directory locations (common for user-installed tools)
    if let Ok(home) = std::env::var("HOME") {
        let home_paths = [
            format!("{}/.local/bin/{}", home, name),
            format!("{}/bin/{}", home, name),
        ];
        for p in home_paths {
            let path = PathBuf::from(&p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_common_executable() {
        // ls should exist on any Linux system
        let result = find_executable("ls");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_find_nonexistent_executable() {
        let result = find_executable("definitely_not_a_real_executable_12345");
        assert!(result.is_none());
    }
}
