//! Installation information implementations for each agent.
//!
//! This module provides platform-specific installation commands for all
//! supported agents. Each function returns an `InstallInfo` with the
//! appropriate commands for the current platform.

use super::{
    InstallInfo, InstallLocation, InstallMethod, Prerequisite, StructuredCommand, VerificationStep,
};

/// Version verification pattern that matches semantic versions.
/// Reuses the same pattern structure from detection/parser.rs.
const VERSION_PATTERN: &str = r"\d+\.\d+\.\d+";

/// Claude Code installation information.
///
/// - Linux/macOS: curl script (native installer)
/// - Windows: PowerShell script (native installer)
/// - Alternative: npm install (requires Node.js 18+)
pub(crate) fn claude_code_install_info() -> InstallInfo {
    #[cfg(windows)]
    let primary = InstallMethod {
        command: StructuredCommand {
            program: "powershell".to_string(),
            args: vec![
                "-Command".to_string(),
                "irm https://claude.ai/install.ps1 | iex".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "irm https://claude.ai/install.ps1 | iex".to_string(),
        description: "Install via PowerShell (native installer)".to_string(),
        location: InstallLocation::UserLocal,
    };

    #[cfg(not(windows))]
    let primary = InstallMethod {
        command: StructuredCommand {
            program: "bash".to_string(),
            args: vec![
                "-c".to_string(),
                "curl -fsSL https://claude.ai/install.sh | bash".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "curl -fsSL https://claude.ai/install.sh | bash".to_string(),
        description: "Install via curl script (native installer)".to_string(),
        location: InstallLocation::UserLocal,
    };

    let npm_alternative = InstallMethod {
        command: StructuredCommand {
            program: "npm".to_string(),
            args: vec![
                "install".to_string(),
                "-g".to_string(),
                "@anthropic-ai/claude-code".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "npm install -g @anthropic-ai/claude-code".to_string(),
        description: "Install via npm (requires Node.js 18+)".to_string(),
        location: InstallLocation::UserLocal,
    };

    InstallInfo {
        primary,
        alternatives: vec![npm_alternative],
        // Native installer has no prerequisites
        prerequisites: vec![],
        verification: VerificationStep {
            command: "claude --version".to_string(),
            expected_pattern: VERSION_PATTERN.to_string(),
            success_message: "Claude Code is installed".to_string(),
        },
        is_supported: true,
        docs_url: "https://docs.anthropic.com/en/docs/claude-code".to_string(),
    }
}

/// Codex installation information.
///
/// - All platforms: npm install (primary)
/// - Note: Windows support is experimental
pub(crate) fn codex_install_info() -> InstallInfo {
    let primary = InstallMethod {
        command: StructuredCommand {
            program: "npm".to_string(),
            args: vec![
                "install".to_string(),
                "-g".to_string(),
                "@openai/codex".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "npm install -g @openai/codex".to_string(),
        description: "Install via npm (Node.js package manager)".to_string(),
        location: InstallLocation::UserLocal,
    };

    let prerequisites = vec![Prerequisite {
        name: "Node.js 18+".to_string(),
        check_command: Some("node --version".to_string()),
        install_url: Some("https://nodejs.org".to_string()),
    }];

    #[cfg(windows)]
    let description_note = " (Windows support is experimental; consider WSL)";
    #[cfg(not(windows))]
    let description_note = "";

    InstallInfo {
        primary,
        alternatives: vec![],
        prerequisites,
        verification: VerificationStep {
            command: "codex --version".to_string(),
            expected_pattern: VERSION_PATTERN.to_string(),
            success_message: format!("Codex is installed{}", description_note),
        },
        is_supported: true,
        docs_url: "https://github.com/openai/codex".to_string(),
    }
}

/// OpenCode installation information.
///
/// - Linux/macOS: curl script (native Go binary)
/// - Windows: scoop install (preferred) or npm
/// - Alternatives: npm install
pub(crate) fn opencode_install_info() -> InstallInfo {
    #[cfg(windows)]
    let primary = InstallMethod {
        command: StructuredCommand {
            program: "scoop".to_string(),
            args: vec!["install".to_string(), "opencode".to_string()],
            env_vars: vec![],
        },
        raw_command: "scoop install opencode".to_string(),
        description: "Install via Scoop (Windows package manager)".to_string(),
        location: InstallLocation::UserLocal,
    };

    #[cfg(not(windows))]
    let primary = InstallMethod {
        command: StructuredCommand {
            program: "bash".to_string(),
            args: vec![
                "-c".to_string(),
                "curl -fsSL https://opencode.ai/install | bash".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "curl -fsSL https://opencode.ai/install | bash".to_string(),
        description: "Install via curl script (native Go binary)".to_string(),
        location: InstallLocation::UserLocal,
    };

    let npm_alternative = InstallMethod {
        command: StructuredCommand {
            program: "npm".to_string(),
            args: vec![
                "install".to_string(),
                "-g".to_string(),
                "opencode-ai@latest".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "npm i -g opencode-ai@latest".to_string(),
        description: "Install via npm (requires Node.js)".to_string(),
        location: InstallLocation::UserLocal,
    };

    // Primary method (curl or scoop) has no prerequisites
    // The npm alternative would need Node.js but we don't list it
    // since it's just an alternative
    let prerequisites = vec![];

    InstallInfo {
        primary,
        alternatives: vec![npm_alternative],
        prerequisites,
        verification: VerificationStep {
            command: "opencode --version".to_string(),
            expected_pattern: VERSION_PATTERN.to_string(),
            success_message: "OpenCode is installed".to_string(),
        },
        is_supported: true,
        docs_url: "https://github.com/anomalyco/opencode".to_string(),
    }
}

/// Gemini CLI installation information.
///
/// - All platforms: npm install (primary)
/// - Requires Node.js 20+ (higher than other agents)
pub(crate) fn gemini_install_info() -> InstallInfo {
    let primary = InstallMethod {
        command: StructuredCommand {
            program: "npm".to_string(),
            args: vec![
                "install".to_string(),
                "-g".to_string(),
                "@google/gemini-cli".to_string(),
            ],
            env_vars: vec![],
        },
        raw_command: "npm install -g @google/gemini-cli".to_string(),
        description: "Install via npm (Node.js package manager)".to_string(),
        location: InstallLocation::UserLocal,
    };

    // Gemini requires Node.js 20+ (higher than other agents)
    let prerequisites = vec![Prerequisite {
        name: "Node.js 20+".to_string(),
        check_command: Some("node --version".to_string()),
        install_url: Some("https://nodejs.org".to_string()),
    }];

    InstallInfo {
        primary,
        alternatives: vec![],
        prerequisites,
        verification: VerificationStep {
            command: "gemini --version".to_string(),
            expected_pattern: VERSION_PATTERN.to_string(),
            success_message: "Gemini CLI is installed".to_string(),
        },
        is_supported: true,
        docs_url: "https://github.com/google-gemini/gemini-cli".to_string(),
    }
}
