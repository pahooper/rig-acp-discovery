//! Type definitions for installation information.
//!
//! This module defines the data structures used to describe how to install
//! AI coding agents on the current platform. The types support both programmatic
//! execution and human-readable display.

use serde::{Deserialize, Serialize};

/// Where an installation method installs to.
///
/// This indicates whether the installation requires elevated privileges
/// and where the binary will be placed.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::InstallLocation;
///
/// let location = InstallLocation::UserLocal;
/// assert_eq!(location, InstallLocation::UserLocal);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallLocation {
    /// User-local installation (no sudo/admin required).
    ///
    /// Examples:
    /// - `~/.local/bin` (Linux)
    /// - `~/AppData/Local/Programs` (Windows)
    /// - npm global with user prefix
    UserLocal,

    /// System-wide installation (may require sudo/admin).
    ///
    /// Examples:
    /// - `/usr/local/bin` (Unix)
    /// - `C:\Program Files` (Windows)
    System,
}

/// A structured command for programmatic execution.
///
/// This provides all the information needed to execute an install command
/// programmatically, separate from the human-readable command string.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::StructuredCommand;
///
/// let cmd = StructuredCommand {
///     program: "npm".to_string(),
///     args: vec!["install".to_string(), "-g".to_string(), "@openai/codex".to_string()],
///     env_vars: vec![],
/// };
/// assert_eq!(cmd.program, "npm");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredCommand {
    /// The program to execute (e.g., "bash", "powershell", "npm").
    pub program: String,

    /// Arguments to pass to the program.
    pub args: Vec<String>,

    /// Environment variables to set before execution (key, value pairs).
    pub env_vars: Vec<(String, String)>,
}

/// A method for installing an agent.
///
/// This includes both the structured command for programmatic use and
/// a raw command string for display/copy-paste.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{InstallMethod, InstallLocation, StructuredCommand};
///
/// let method = InstallMethod {
///     command: StructuredCommand {
///         program: "npm".to_string(),
///         args: vec!["install".to_string(), "-g".to_string(), "@openai/codex".to_string()],
///         env_vars: vec![],
///     },
///     raw_command: "npm install -g @openai/codex".to_string(),
///     description: "Install via npm (Node.js package manager)".to_string(),
///     location: InstallLocation::UserLocal,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMethod {
    /// Structured command for programmatic execution.
    pub command: StructuredCommand,

    /// Raw command string for display/copy-paste.
    pub raw_command: String,

    /// Human-readable description (e.g., "Install via npm").
    pub description: String,

    /// Where this method installs to.
    pub location: InstallLocation,
}

/// A prerequisite for installation.
///
/// Some agents require other software to be installed first (e.g., Node.js).
/// This struct provides information about checking and installing prerequisites.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::Prerequisite;
///
/// let prereq = Prerequisite {
///     name: "Node.js 18+".to_string(),
///     check_command: Some("node --version".to_string()),
///     install_url: Some("https://nodejs.org".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    /// What is required (e.g., "Node.js 18+", "npm").
    pub name: String,

    /// Command to check if the prerequisite is met (e.g., "node --version").
    pub check_command: Option<String>,

    /// URL for installing this prerequisite.
    pub install_url: Option<String>,
}

/// A step to verify successful installation.
///
/// After installation, this step can be used to confirm the agent
/// was installed correctly.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::VerificationStep;
///
/// let verify = VerificationStep {
///     command: "claude --version".to_string(),
///     expected_pattern: r"\d+\.\d+\.\d+".to_string(),
///     success_message: "Claude Code is installed".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    /// Command to run for verification (e.g., "claude --version").
    pub command: String,

    /// Regex pattern the output should match for success.
    pub expected_pattern: String,

    /// Human-readable success message.
    pub success_message: String,
}

/// Complete installation information for an agent.
///
/// This struct contains everything needed to install an agent:
/// - The primary (recommended) installation method
/// - Alternative installation methods
/// - Prerequisites that must be installed first
/// - How to verify successful installation
/// - Whether the agent is supported on the current platform
///
/// # Example
///
/// ```rust,no_run
/// use rig_acp_discovery::AgentKind;
///
/// let info = AgentKind::Codex.install_info();
/// println!("Install with: {}", info.primary.raw_command);
/// for prereq in &info.prerequisites {
///     println!("Requires: {}", prereq.name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallInfo {
    /// The recommended installation method for this platform.
    pub primary: InstallMethod,

    /// Alternative installation methods (e.g., npm when native is primary).
    pub alternatives: Vec<InstallMethod>,

    /// Prerequisites that must be installed first.
    pub prerequisites: Vec<Prerequisite>,

    /// How to verify successful installation.
    pub verification: VerificationStep,

    /// Whether this agent is supported on the current platform.
    ///
    /// If `false`, the install commands are provided for informational
    /// purposes but may not work correctly.
    pub is_supported: bool,

    /// URL to official documentation for this agent.
    pub docs_url: String,
}
