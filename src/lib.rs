//! kittyaudio is an audio library focusing on simplicity.
//!
//! # Example
//!
//! ```rust
//! use kittyaudio::{include_sound, Mixer};
//!
//! fn main() {
//!     // include a sound into the executable.
//!     // this type can be cheaply cloned.
//!     let sound = include_sound!("jump.ogg").unwrap();
//!
//!     // create sound mixer
//!     let mut mixer = Mixer::new();
//!     mixer.init(); // use init_ex to specify settings
//!
//!     let playing_sound = mixer.play();
//!     playing_sound.set_volume(0.5); // decrease volume
//!
//!     mixer.wait(); // wait for all sounds to finish
//! }
//! ```
//!
//! See more examples in the `examples` directory.
//!
//! # Features
//!
//! * Low-latency audio playback
//! * Cross-platform audio playback (including wasm)
//! * Handle device changes or disconnects in real time
//! * Low CPU usage
//! * Minimal dependencies
//! * Minimal memory allocations
//! * No `panic!()` or `.unwrap()`, always propogate errors
//! * No unsafe code
//! * Simple API, while being customizable
//! * Optionally use [Symphonia](https://github.com/pdeljanov/Symphonia) to support most audio formats
//! * Feature to disable audio playback support, if you want to use kittyaudio purely as an audio library
//! * Commands to change volume, playback rate and position in the sound with easings
//!
//! # Roadmap
//!
//! Those features are not implemented yet.
//!
//! * Effects (reverb, delay, eq, etc.)
//! * C API
//! * Audio streaming from disk

#![warn(missing_docs)] // warn on missing function docs

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
