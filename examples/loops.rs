use kittyaudio::{include_sound, Change, Command, Easing, Mixer, PlaybackRate};

fn main() {
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    let mut mixer = Mixer::new();
    mixer.init();

    let sound = mixer.play(sound);

    // set the loop points to be from 2 to 4 seconds
    sound.set_loop_enabled(true);
    sound.set_loop(2.0..=4.0);

    // after 6 seconds (the loop has ran 1 time), change the loop region to
    // 4.0..=4.1 seconds in the span on 5 seconds
    let command = Command::new(Change::LoopSeconds(4.0..=4.1), Easing::Linear, 6.0, 5.0);
    sound.add_command(command);

    // reverse the sound gradually after 11 seconds upto 16 seconds (11+5)
    let command = Command::new(
        Change::PlaybackRate(PlaybackRate::Factor(-1.0)),
        Easing::Linear,
        11.0,
        5.0,
    );
    sound.add_command(command);

    // change the loop to be 4-5 seconds from 13 secons in the span of 5 seconds
    // (the playback rate change command is applied simultaneously)
    let command = Command::new(Change::LoopSeconds(4.0..=6.0), Easing::Linear, 13.0, 5.0);
    sound.add_command(command);

    mixer.wait();
}
