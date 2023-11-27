//! Simple example of playing a sound with the [`kittyaudio`] library.
//!
//! This example requires the `use-symphonia` feature.

use kittyaudio::{include_sound, Mixer};
use std::time::Instant;

fn main() {
    println!("decoding...");
    let start = Instant::now();

    // include a sound in the executable
    // this is a shorthand for `Sound::from_cursor(Cursor::new(include_bytes!("sound.mp3")))`
    // song credit: https://modarchive.org/member.php?84702
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    // the [`Sound`] type will share the same sound data between all clones,
    // so you can clone `sound` cheaply.

    // decoding speed is really fast in release mode (thanks to Symphonia)
    println!(
        "decoded in {:?}, sound length: {:?}",
        start.elapsed(),
        sound.duration()
    );

    // create the sound mixer and start the audio thread
    let mut mixer = Mixer::new();
    mixer.init(); // use init_ex to specify settings

    // play the sound
    let _playing_sound = mixer.play(sound.clone());

    // playing_sound.set_playback_rate(PlaybackRate::Factor(1.5)); // - try this too!
    // playing_sound.set_volume(1.0);

    // you can clone and share the returned `playing_sound`, data will be shared
    // between all clones.

    // wait for all sounds to finish (use mixer.is_finished() to check for that)
    mixer.wait();
}

// same thing but without comments.
#[cfg(not)]
fn main() {
    let sound = include_sound!("drozerix_-_crush.ogg").unwrap();

    let mut mixer = Mixer::new();
    mixer.init();

    let _playing_sound = mixer.play(sound);
    mixer.wait();
}
