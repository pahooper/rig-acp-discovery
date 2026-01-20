//! # rig-acp-discovery
//!
//! Agent discovery and installation for AI coding agents (Claude Code, Codex, OpenCode, Gemini).
//!
//! This crate provides types and functions for detecting installed AI coding agents
//! and their capabilities, as well as programmatic installation with progress reporting.
//! It can be used independently or integrated with rig-acp via the `discovery` feature flag.
//!
//! ## Features
//!
//! - `AgentKind` enum identifying supported agents
//! - `AgentStatus` enum representing detection results with rich metadata
//! - `DetectOptions` struct for configuring detection timeout
//! - `detect()` async function for detecting a single agent
//! - `detect_all()` async function for detecting all agents in parallel
//! - `can_install()` async function for prerequisite checking
//! - `install()` async function for programmatic installation with progress
//!
//! ## Detection Example
//!
//! ```rust,no_run
//! use rig_acp_discovery::{AgentKind, DetectOptions, detect, detect_all};
//! use std::time::Duration;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     // Detect a single agent with default options
//!     let status = detect(AgentKind::ClaudeCode).await;
//!     if status.is_usable() {
//!         println!("Claude Code is installed at {:?}", status.path());
//!     }
//!
//!     // Detect all agents in parallel
//!     let all_agents = detect_all().await;
//!     for (kind, result) in all_agents {
//!         match result {
//!             Ok(status) if status.is_usable() => {
//!                 println!("{}: available", kind.display_name());
//!             }
//!             Ok(_) => {
//!                 println!("{}: not available", kind.display_name());
//!             }
//!             Err(e) => {
//!                 println!("{}: detection failed: {}", kind.display_name(), e.description());
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Installation
//!
//! Agents can be installed programmatically:
//!
//! ```rust,no_run
//! use rig_acp_discovery::{AgentKind, InstallOptions, InstallProgress, install, can_install};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     // Pre-flight check
//!     if let Err(e) = can_install(AgentKind::Codex).await {
//!         println!("Cannot install: {}. Fix: {}", e, e.fix_suggestion());
//!         return;
//!     }
//!
//!     // Install with progress
//!     let result = install(
//!         AgentKind::Codex,
//!         InstallOptions::default(),
//!         |progress| match progress {
//!             InstallProgress::Started { agent } => println!("Installing {}...", agent.display_name()),
//!             InstallProgress::Completed { agent } => println!("{} installed!", agent.display_name()),
//!             _ => {}
//!         },
//!     ).await;
//!
//!     if let Err(e) = result {
//!         println!("Installation failed: {}. Fix: {}", e, e.fix_suggestion());
//!     }
//! }
//! ```

mod agent_kind;
mod agent_status;
mod detect;
mod detection;
mod install;
mod options;

pub use agent_kind::AgentKind;
pub use agent_status::{AgentStatus, DetectionError, InstalledMetadata};
pub use detect::{detect, detect_all, detect_all_with_options, detect_with_options};
pub use install::{
    can_install, install, InstallError, InstallInfo, InstallLocation, InstallMethod,
    InstallOptions, InstallProgress, Prerequisite, StructuredCommand, VerificationStep,
};
pub use options::DetectOptions;
