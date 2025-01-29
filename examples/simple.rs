//! Simple example of playing a sound with the [`kittyaudio`] library.
//!
//! This example requires the `symphonia` feature.

use kittyaudio::{include_sound, Mixer};

fn main() {
    // include a sound in the executable
    // this is a shorthand for `Sound::from_cursor(Cursor::new(include_bytes!("sound.mp3")))`
    // song credit: https://modarchive.org/member.php?84702
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    // the [`Sound`] type will share the same sound data between all clones,
    // so you can clone `sound` cheaply.

    // create the sound mixer and start the audio thread
    let mut mixer = Mixer::new();
    mixer.init(); // use init_ex to specify settings

    // play the sound
    mixer.play(sound);

    // wait for all sounds to finish (use mixer.is_finished() to check for that)
    mixer.wait();
}
