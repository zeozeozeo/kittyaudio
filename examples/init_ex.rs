//! Initialize mixer with extra stream parameters.
use kittyaudio::{include_sound, Device, Mixer, StreamSettings};

fn main() {
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();

    let mut mixer = Mixer::new();

    // if we set any value to None, the default one will be used
    let settings = StreamSettings {
        buffer_size: Some(0),
        ..Default::default()
    };

    // start the mixer with the default device and 0 buffer size.
    // see the Device struct for more details.
    mixer.init_ex(Device::Default, settings);

    let sound = mixer.play(sound);
    sound.seek_by(50.0); // seek 50 seconds forward

    mixer.wait(); // wait for all sounds to finish
}
