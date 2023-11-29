use kittyaudio::{include_sound, Change, Command, Easing, Mixer, PlaybackRate};

fn main() {
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    let mut mixer = Mixer::new();
    mixer.init();

    let sound = mixer.play(sound);
    sound.seek_by(5.0);

    // after the sound has played for 1 second, gradually decrease volume
    // in the duration of 5 seconds
    let command = Command::new(Change::Volume(0.0), Easing::Linear, 1.0, 5.0);
    sound.add_command(command);

    // after 6 seconds, increase volume for 3 seconds with exponential out easing
    let command = Command::new(Change::Volume(1.0), Easing::ExpoOut, 6.0, 3.0);
    sound.add_command(command);

    // after 9 seconds, slow down the playback rate to 0.3 in 6 seconds
    let command = Command::new(
        Change::PlaybackRate(PlaybackRate::Factor(0.3)),
        Easing::Linear,
        9.0,
        6.0,
    );
    sound.add_command(command);

    // after 18 seconds, make the sound play 2x faster with elastic in out easing
    let command = Command::new(
        Change::PlaybackRate(PlaybackRate::Factor(2.0)),
        Easing::ElasticInOut,
        18.0,
        5.0,
    );
    sound.add_command(command);

    mixer.wait(); // wait until the sound is finished
}
