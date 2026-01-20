//! Installation information and execution for AI coding agents.
//!
//! This module provides:
//! - Platform-appropriate installation instructions via `AgentKind::install_info()`
//! - Pre-flight prerequisite checking via `can_install()`
//! - Progress reporting types for installation UI
//! - Error types with actionable fix suggestions
//!
//! # Pre-flight Check Example
//!
//! ```rust,no_run
//! use rig_acp_discovery::{AgentKind, can_install};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     match can_install(AgentKind::Codex).await {
//!         Ok(()) => println!("Ready to install Codex"),
//!         Err(e) => {
//!             println!("Cannot install: {}", e);
//!             println!("Fix: {}", e.fix_suggestion());
//!         }
//!     }
//! }
//! ```
//!
//! # Installation Info Example
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

mod errors;
pub(crate) mod info;
mod prereq;
mod progress;
mod types;

pub use errors::InstallError;
pub use prereq::can_install;
pub use progress::{InstallOptions, InstallProgress};
pub use types::{
    InstallInfo, InstallLocation, InstallMethod, Prerequisite, StructuredCommand, VerificationStep,
};
