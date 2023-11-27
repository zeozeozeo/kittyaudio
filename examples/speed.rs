//! Control the speed of a sound.
use kittyaudio::{include_sound, Mixer, PlaybackRate};

fn main() {
    let mut line = String::new();
    println!("--------------- enter playback speed: ");
    std::io::stdin().read_line(&mut line).unwrap();

    let speed: f64 = line.trim().parse().unwrap();

    // load sound
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    // create mixer
    let mut mixer = Mixer::new();
    mixer.init();

    let sound = mixer.play(sound);

    if speed.is_sign_negative() {
        println!("speed is negative, playing backwards");
        sound.seek_to_end();
    }

    // use PlaybackRate::Factor(speed) to modify speed and
    // PlaybackRate::Semitones(semitones) to modify semitones.
    sound.set_playback_rate(PlaybackRate::Factor(speed));
    mixer.wait(); // wait for all sounds to finish
}
