//! KittyAudio is a Rust audio library focusing on simplicity.
#![warn(missing_docs)]

#[cfg(feature = "playback")]
mod backend;

mod command;
mod error;
mod mixer;
mod resampler;
mod sound;

#[cfg(feature = "playback")]
pub use backend::*;

pub use command::*;
pub use error::*;
pub use mixer::*;
pub use resampler::*;
pub use sound::*;

// Re-export the cpal and symphonia crate
#[cfg(feature = "playback")]
pub use cpal;

#[cfg(feature = "use-symphonia")]
pub use symphonia;
