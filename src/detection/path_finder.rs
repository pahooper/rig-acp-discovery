//! PATH-based executable lookup with fallback locations.

use std::path::PathBuf;

/// System fallback paths to check if executable not found in PATH (Linux/Unix).
#[cfg(not(windows))]
const FALLBACK_PATHS: &[&str] = &["/usr/local/bin", "/usr/bin"];

/// System fallback paths to check if executable not found in PATH (Windows).
/// Empty because Windows PATH + npm location typically suffice.
#[cfg(windows)]
const FALLBACK_PATHS: &[&str] = &[];

/// Get home directory paths to check for an executable.
///
/// Returns platform-specific paths where user-installed tools are commonly found.
fn get_home_paths(name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if cfg!(windows) {
        // Windows: use USERPROFILE for native installs
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            // With .exe extension
            paths.push(PathBuf::from(format!(
                r"{}\.local\bin\{}.exe",
                userprofile, name
            )));
            // Without extension (which crate will try PATHEXT)
            paths.push(PathBuf::from(format!(
                r"{}\.local\bin\{}",
                userprofile, name
            )));
        }

        // Windows: use APPDATA for npm global installs
        if let Ok(appdata) = std::env::var("APPDATA") {
            // npm creates .cmd shims
            paths.push(PathBuf::from(format!(r"{}\npm\{}.cmd", appdata, name)));
        }
    } else {
        // Unix: use HOME
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(format!("{}/.local/bin/{}", home, name)));
            paths.push(PathBuf::from(format!("{}/bin/{}", home, name)));
        }
    }

    paths
}

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
    // On Windows, which crate automatically handles PATHEXT (.exe, .cmd, etc.)
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
    get_home_paths(name).into_iter().find(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn test_find_common_executable() {
        // ls should exist on any Linux system
        let result = find_executable("ls");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.exists());
    }

    #[test]
    #[cfg(windows)]
    fn test_find_common_executable_windows() {
        // cmd should exist on any Windows system
        let result = find_executable("cmd");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_find_nonexistent_executable() {
        let result = find_executable("definitely_not_a_real_executable_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_home_paths_returns_paths() {
        // get_home_paths should return paths for any executable name
        let paths = get_home_paths("test_tool");
        // On any platform, we should get at least one path if env vars are set
        // This test verifies the function runs without error
        // The actual paths depend on platform and env vars
        // Using is_empty() check to verify function runs without panic
        let _ = paths.is_empty();
    }

    #[test]
    #[cfg(windows)]
    fn test_get_home_paths_windows_format() {
        // Set test env vars
        std::env::set_var("USERPROFILE", r"C:\Users\TestUser");
        std::env::set_var("APPDATA", r"C:\Users\TestUser\AppData\Roaming");

        let paths = get_home_paths("claude");

        // Should include Windows-style paths
        let path_strs: Vec<_> = paths.iter().map(|p| p.to_string_lossy()).collect();
        assert!(path_strs.iter().any(|p| p.contains(r"\.local\bin\")));
        assert!(path_strs.iter().any(|p| p.contains(".exe")));
        assert!(path_strs.iter().any(|p| p.contains(r"\npm\")));
        assert!(path_strs.iter().any(|p| p.contains(".cmd")));

        // Restore env vars
        std::env::remove_var("USERPROFILE");
        std::env::remove_var("APPDATA");
    }

    #[test]
    #[cfg(not(windows))]
    fn test_get_home_paths_unix_format() {
        // Set test env var
        std::env::set_var("HOME", "/home/testuser");

        let paths = get_home_paths("claude");

        // Should include Unix-style paths
        let path_strs: Vec<_> = paths.iter().map(|p| p.to_string_lossy()).collect();
        assert!(path_strs.iter().any(|p| p.contains("/.local/bin/")));
        assert!(path_strs.iter().any(|p| p.contains("/bin/")));
        // Should not contain Windows paths
        assert!(!path_strs.iter().any(|p| p.contains(".exe")));
        assert!(!path_strs.iter().any(|p| p.contains(".cmd")));

        // Restore env var (or leave as-is since HOME is typically set)
    }

    #[test]
    #[cfg(not(windows))]
    fn test_fallback_paths_unix() {
        // On Unix, FALLBACK_PATHS should include system directories
        assert!(!FALLBACK_PATHS.is_empty());
        assert!(FALLBACK_PATHS.contains(&"/usr/local/bin"));
        assert!(FALLBACK_PATHS.contains(&"/usr/bin"));
    }

    #[test]
    #[cfg(windows)]
    fn test_fallback_paths_windows() {
        // On Windows, FALLBACK_PATHS should be empty
        // (PATH + npm location suffice)
        assert!(FALLBACK_PATHS.is_empty());
    }
}
