//! Installation execution for AI coding agents.
//!
//! This module provides:
//! - [`can_install`] - Pre-flight check for prerequisites
//! - [`install`] - Programmatic installation with progress reporting
//! - [`InstallError`] - Error types with actionable fix suggestions
//! - [`InstallProgress`] - Progress stages for UI updates
//! - [`InstallOptions`] - Configuration (timeout, etc.)
//!
//! # Consent Model
//!
//! Calling `install()` IS consent to install. The library does not prompt
//! for confirmation. Your application's UI should confirm with the user
//! before calling `install()`.
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
//! # Installation Example
//!
//! ```rust,no_run
//! use rig_acp_discovery::{AgentKind, install, can_install, InstallOptions};
//!
//! async fn install_agent() {
//!     // Check prerequisites first (optional but recommended for UI)
//!     can_install(AgentKind::ClaudeCode).await.unwrap();
//!
//!     // Install with progress callback
//!     install(
//!         AgentKind::ClaudeCode,
//!         InstallOptions::default(),
//!         |p| println!("{:?}", p),
//!     ).await.unwrap();
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
mod executor;
pub(crate) mod info;
mod prereq;
mod progress;
mod types;

pub use errors::InstallError;
pub use executor::install;
pub use prereq::can_install;
pub use progress::{InstallOptions, InstallProgress};
pub use types::{
    InstallInfo, InstallLocation, InstallMethod, Prerequisite, StructuredCommand, VerificationStep,
};
