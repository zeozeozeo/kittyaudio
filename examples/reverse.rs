//! Reverse audio.
use kittyaudio::{include_sound, Mixer};

fn main() {
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    let mut mixer = Mixer::new();
    mixer.init();

    let sound = mixer.play(sound);
    sound.seek_to_end();
    sound.reverse(); // or sound.set_playback_rate(PlaybackRate::Factor(-1.0))

    mixer.wait(); // wait for all sounds to finish
}
