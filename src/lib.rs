//! Spaniel Interactive Deserialiser
//! --------------------------------
//!
//! This crate is a Rust library which uses the Serde serialisation framework
//! to capture data interactively from users.

mod error;
mod internal;
mod u8i8;
mod util;

/// Serde deserialiser.
pub mod de;
/// Traits and decorators for working with prompts.
pub mod prompt;
#[cfg(feature = "rustyline")]
/// Prompt based on the RustyLine crate.
pub mod rustyline;
/// Serde serialiser.
pub mod ser;
#[cfg(feature = "stdio")]
/// Prompt based on `std::io`.
pub mod stdio;

pub use error::{Error, Result, UserAction};
pub use util::{
  from_bare_prompt, from_bare_prompt_confirm, from_console, from_prompt,
  from_replay_prompt, to_prompt,
};
