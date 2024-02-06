//! Audio panning using commands and Sound functions.
use kittyaudio::{include_sound, Change, Command, Easing, Mixer};

fn main() {
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();
    let mut mixer = Mixer::new();
    mixer.init();
    let sound = mixer.play(sound);

    // 0.0 = hard left
    // 0.5 = center (default)
    // 1.0 = hard right
    sound.set_panning(1.0); // hard right

    // after 1 second, change the panning to hard left in the interval of 3 seconds
    sound.add_command(Command::new(Change::Panning(0.0), Easing::Linear, 1.0, 3.0));

    // after 5 seconds, change the panning to center in the interval of 2 seconds with bounce-out easing
    sound.add_command(Command::new(
        Change::Panning(0.5),
        Easing::BounceOut,
        5.0,
        2.0,
    ));

    // after 9 seconds, change the panning to hard right immediately
    sound.add_command(Command::new(Change::Panning(1.0), Easing::Linear, 9.0, 0.0));

    // print the panning each 0.1s
    while !mixer.is_finished() {
        println!("panning: {}", sound.panning());
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
