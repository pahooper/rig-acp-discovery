//! Agent detection functions.

use crate::{AgentKind, AgentStatus};
use futures::future::join_all;
use std::collections::HashMap;

/// Detect a single agent by kind.
///
/// This function checks if the specified agent is installed and usable.
/// It searches the system PATH for the agent's executable and verifies
/// its availability.
///
/// # Phase 1 Implementation
///
/// In Phase 1, this function returns `AgentStatus::NotInstalled` for all agents.
/// This is the correct behavior until Phase 2 adds actual PATH detection logic.
/// The function signature and return type establish the API contract.
///
/// # Arguments
///
/// * `kind` - The type of agent to detect
///
/// # Returns
///
/// An `AgentStatus` representing the detection result:
/// - `Installed(metadata)` - Agent found and usable
/// - `NotInstalled` - Agent not found
/// - `VersionMismatch { .. }` - Agent found but version incompatible
/// - `Unknown { .. }` - Detection failed with error
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{AgentKind, AgentStatus, detect};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let status = detect(AgentKind::ClaudeCode).await;
///     if status.is_usable() {
///         println!("Claude Code is available at {:?}", status.path());
///     }
/// }
/// ```
pub async fn detect(_kind: AgentKind) -> AgentStatus {
    // Phase 1: Return NotInstalled for all agents
    // Phase 2 will add actual PATH detection logic
    AgentStatus::NotInstalled
}

/// Detect all known agents in parallel.
///
/// This function detects all agents defined in `AgentKind` concurrently,
/// returning a map of agent kinds to their detection status.
///
/// # Performance
///
/// Detection is performed in parallel using `futures::future::join_all`,
/// so the total detection time is approximately the time of the slowest
/// agent detection, not the sum of all detection times.
///
/// # Returns
///
/// A `HashMap` mapping each `AgentKind` to its `AgentStatus`.
///
/// # Example
///
/// ```rust
/// use rig_acp_discovery::{AgentKind, detect_all};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let all = detect_all().await;
///
///     for (kind, status) in &all {
///         println!("{}: {}", kind.display_name(),
///             if status.is_usable() { "available" } else { "not available" });
///     }
///
///     // Check specific agent
///     if let Some(status) = all.get(&AgentKind::ClaudeCode) {
///         if status.is_usable() {
///             println!("Claude Code ready!");
///         }
///     }
/// }
/// ```
pub async fn detect_all() -> HashMap<AgentKind, AgentStatus> {
    let futures: Vec<_> = AgentKind::all()
        .map(|kind| async move { (kind, detect(kind).await) })
        .collect();

    join_all(futures).await.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_returns_not_installed() {
        // Phase 1: All agents return NotInstalled
        let status = detect(AgentKind::ClaudeCode).await;
        assert!(!status.is_usable());
        assert!(!status.is_installed());
        assert!(matches!(status, AgentStatus::NotInstalled));
    }

    #[tokio::test]
    async fn test_detect_all_returns_all_agents() {
        let all = detect_all().await;

        // Should have an entry for each agent kind
        assert_eq!(all.len(), 4);
        assert!(all.contains_key(&AgentKind::ClaudeCode));
        assert!(all.contains_key(&AgentKind::Codex));
        assert!(all.contains_key(&AgentKind::OpenCode));
        assert!(all.contains_key(&AgentKind::Gemini));

        // All should be NotInstalled in Phase 1
        for (_, status) in &all {
            assert!(matches!(status, AgentStatus::NotInstalled));
        }
    }

    #[tokio::test]
    async fn test_detect_all_parallel_execution() {
        // This test verifies the function completes (parallel execution works)
        // Actual parallel timing would require real I/O
        let all = detect_all().await;
        assert!(!all.is_empty());
    }
}
