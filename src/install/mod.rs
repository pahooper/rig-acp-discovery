//! Installation information for AI coding agents.
//!
//! This module provides platform-appropriate installation instructions for
//! agents that are not currently installed. Use `AgentKind::install_info()`
//! to get installation details for any agent.
//!
//! # Example
//!
//! ```rust,no_run
//! use rig_acp_discovery::AgentKind;
//!
//! let info = AgentKind::ClaudeCode.install_info();
//! println!("To install Claude Code, run:");
//! println!("  {}", info.primary.raw_command);
//!
//! if !info.prerequisites.is_empty() {
//!     println!("\nPrerequisites:");
//!     for prereq in &info.prerequisites {
//!         println!("  - {}", prereq.name);
//!     }
//! }
//!
//! println!("\nVerify installation:");
//! println!("  {}", info.verification.command);
//! ```

mod types;

pub use types::{
    InstallInfo, InstallLocation, InstallMethod, Prerequisite, StructuredCommand, VerificationStep,
};
