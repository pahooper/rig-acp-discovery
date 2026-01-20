//! Detection implementation submodule.
//!
//! This module contains the internal implementation details for detecting
//! AI coding agents on the system. It provides:
//!
//! - `find_executable`: PATH-based executable lookup with fallbacks
//! - `check_version`: Async version check with 2-second timeout
//! - `parse_version`: Regex-based version extraction from CLI output

mod parser;
mod path_finder;
mod version;

pub(crate) use parser::parse_version;
pub(crate) use path_finder::find_executable;
pub(crate) use version::check_version;
