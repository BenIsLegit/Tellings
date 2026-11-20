//! The engine behind Prayer Video Creator.
//!
//! The app and the command-line tool are both thin shells over this crate, so a
//! video made by clicking and a video made by asking Claude Code or Codex come
//! out the same.

pub mod beads;
pub mod error;
pub mod plan;
pub mod prayers;
pub mod setup;

pub use error::{Error, Result};
