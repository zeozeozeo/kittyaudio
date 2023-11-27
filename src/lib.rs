//! KittyAudio is a Rust audio library focusing on simplicity.
#![warn(missing_docs)]

mod backend;
mod error;
mod mixer;
mod resampler;
mod sound;

pub use backend::*;
pub use error::*;
pub use mixer::*;
pub use resampler::*;
pub use sound::*;

// Re-export the cpal and symphonia crate
pub use cpal;

#[cfg(feature = "use-symphonia")]
pub use symphonia;
