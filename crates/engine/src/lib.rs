//! The engine behind Prayer Video Creator.
//!
//! The app and the command-line tool are both thin shells over this crate, so a
//! video made by clicking and a video made by asking Claude Code or Codex come
//! out the same.
//!
//! The path a video takes through the modules:
//!
//! ```text
//! library  find a setup by name          setup   fill in every default
//! plan     list every scene, pick the narrowest rule for each     (variety: whose turn it is)
//! speech   record what is missing, bring it to pace      (elevenlabs, secrets)
//! audio    lay the prayers end to end
//! frames   draw a still for each scene                    (cdp, renderer/frame.js)
//! video    hold each still for its sound, join, add sound
//! make     run all of the above, and refuse to spend credits unasked
//! ```

pub mod audio;
pub mod beads;
pub mod cdp;
pub mod elevenlabs;
pub mod error;
pub mod frames;
pub mod library;
pub mod make;
pub mod paths;
pub mod plan;
pub mod prayers;
pub mod reference;
pub mod secrets;
pub mod setup;
pub mod speech;
pub mod tools;
pub mod variety;
pub mod video;

pub use error::{Error, Result};
