//! # rig-acp-discovery
//!
//! Agent discovery for AI coding agents (Claude Code, Codex, OpenCode, Gemini).
//!
//! This crate provides types and functions for detecting installed AI coding agents
//! and their capabilities. It can be used independently or integrated with rig-acp
//! via the `discovery` feature flag.
//!
//! ## Features
//!
//! - `AgentKind` enum identifying supported agents
//! - `AgentStatus` enum representing detection results with rich metadata
//! - `detect()` async function for detecting a single agent
//! - `detect_all()` async function for detecting all agents in parallel
//!
//! ## Example
//!
//! ```rust,no_run
//! use rig_acp_discovery::{AgentKind, detect, detect_all};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     // Detect a single agent
//!     let status = detect(AgentKind::ClaudeCode).await;
//!     if status.is_usable() {
//!         println!("Claude Code is installed at {:?}", status.path());
//!     }
//!
//!     // Detect all agents in parallel
//!     let all_agents = detect_all().await;
//!     for (kind, status) in all_agents {
//!         println!("{}: usable={}", kind.display_name(), status.is_usable());
//!     }
//! }
//! ```

mod agent_kind;
mod agent_status;
mod detect;

pub use agent_kind::AgentKind;
pub use agent_status::{AgentStatus, DetectionError, InstalledMetadata};
pub use detect::{detect, detect_all};
