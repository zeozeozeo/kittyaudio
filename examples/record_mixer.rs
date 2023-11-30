//! Demonstrates using a custom mixer for recording audio.
//! This example does NOT require the `cpal` feature.

use std::time::Instant;

use kittyaudio::{include_sound, Frame, RecordMixer};

fn main() {
    println!("loading sound...");
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();
    let mut mixer = RecordMixer::new();

    mixer.play(sound);

    println!("rendering audio...");
    let start = Instant::now();

    // make a buffer where we will store the samples
    let mut buffer = [Frame::ZERO; 4096]; // 32 kib

    while !mixer.is_finished() {
        mixer.fill_buffer(44100, &mut buffer);

        // we recorded 4096 audio samples in an audio buffer, do something with them...
    }

    println!("rendered in {:?}", start.elapsed());
}
