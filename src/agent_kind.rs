//! Agent kind enum identifying supported AI coding agents.

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

/// The type of AI coding agent.
///
/// This enum identifies the supported AI coding agents that can be detected
/// and used with rig-acp. Each variant corresponds to a specific CLI tool
/// that implements the ACP protocol.
///
/// # Extensibility
///
/// This enum is marked `#[non_exhaustive]` to allow adding new agent types
/// in future versions. When matching on `AgentKind`, always include a wildcard
/// pattern to handle future variants:
///
/// ```rust
/// use rig_acp_discovery::AgentKind;
///
/// fn handle_agent(kind: AgentKind) {
///     match kind {
///         AgentKind::ClaudeCode => println!("Claude Code"),
///         AgentKind::Codex => println!("Codex"),
///         AgentKind::OpenCode => println!("OpenCode"),
///         AgentKind::Gemini => println!("Gemini"),
///         _ => println!("Unknown agent type"),
///     }
/// }
/// ```
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::AgentKind;
///
/// // Iterate over all known agents
/// for kind in AgentKind::all() {
///     println!("{}: {}", kind.display_name(), kind.executable_name());
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumIter)]
#[non_exhaustive]
pub enum AgentKind {
    /// Anthropic's Claude Code agent (claude CLI)
    ClaudeCode,
    /// Zed's Codex agent (codex CLI)
    Codex,
    /// OpenCode agent (opencode CLI)
    OpenCode,
    /// Google's Gemini agent (gemini CLI)
    Gemini,
}

impl AgentKind {
    /// The executable name to search for in PATH.
    ///
    /// This is the command name that should be invoked to run the agent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentKind;
    ///
    /// assert_eq!(AgentKind::ClaudeCode.executable_name(), "claude");
    /// assert_eq!(AgentKind::Codex.executable_name(), "codex");
    /// ```
    pub fn executable_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
        }
    }

    /// Human-readable display name for the agent.
    ///
    /// This is a friendly name suitable for display in UIs and messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentKind;
    ///
    /// assert_eq!(AgentKind::ClaudeCode.display_name(), "Claude Code");
    /// assert_eq!(AgentKind::Gemini.display_name(), "Gemini CLI");
    /// ```
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::OpenCode => "OpenCode",
            Self::Gemini => "Gemini CLI",
        }
    }

    /// Iterator over all known agent kinds.
    ///
    /// This is useful for detecting all agents or building selection UIs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rig_acp_discovery::AgentKind;
    ///
    /// let agents: Vec<_> = AgentKind::all().collect();
    /// assert_eq!(agents.len(), 4);
    /// ```
    pub fn all() -> impl Iterator<Item = Self> {
        <Self as IntoEnumIterator>::iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executable_names() {
        assert_eq!(AgentKind::ClaudeCode.executable_name(), "claude");
        assert_eq!(AgentKind::Codex.executable_name(), "codex");
        assert_eq!(AgentKind::OpenCode.executable_name(), "opencode");
        assert_eq!(AgentKind::Gemini.executable_name(), "gemini");
    }

    #[test]
    fn test_display_names() {
        assert_eq!(AgentKind::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(AgentKind::Codex.display_name(), "Codex");
        assert_eq!(AgentKind::OpenCode.display_name(), "OpenCode");
        assert_eq!(AgentKind::Gemini.display_name(), "Gemini CLI");
    }

    #[test]
    fn test_all_iterator() {
        let all: Vec<_> = AgentKind::all().collect();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&AgentKind::ClaudeCode));
        assert!(all.contains(&AgentKind::Codex));
        assert!(all.contains(&AgentKind::OpenCode));
        assert!(all.contains(&AgentKind::Gemini));
    }

    #[test]
    fn test_derives() {
        // Test Clone
        let kind = AgentKind::ClaudeCode;
        let cloned = kind;
        assert_eq!(kind, cloned);

        // Test Hash (via HashSet)
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AgentKind::ClaudeCode);
        set.insert(AgentKind::Codex);
        assert_eq!(set.len(), 2);

        // Test Serialize/Deserialize
        let json = serde_json::to_string(&AgentKind::ClaudeCode).unwrap();
        let deserialized: AgentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AgentKind::ClaudeCode);
    }
}
