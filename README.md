# 🐱 kittyaudio

![docs.rs](https://img.shields.io/docsrs/kittyaudio) ![Downloads on Crates.io](https://img.shields.io/crates/d/kittyaudio)

#### [crates.io](https://crates.io/crates/kittyaudio) | [docs.rs](https://docs.rs/kittyaudio/0.1.0/kittyaudio/) | [examples](https://github.com/zeozeozeo/kittyaudio/tree/master/examples)

kittyaudio is a Rust audio playback library focusing on simplicity, speed and low-latency audio playback.

Installation with `cargo`:

```
cargo add kittyaudio
```

# Example

```rust
use kittyaudio::{include_sound, Mixer};

fn main() {
    // include a sound into the executable.
    // this type can be cheaply cloned.
    let sound = include_sound!("jump.ogg").unwrap();

    // create sound mixer
    let mut mixer = Mixer::new();
    mixer.init(); // use init_ex to specify settings

    let playing_sound = mixer.play();
    playing_sound.set_volume(0.5); // decrease volume

    mixer.wait(); // wait for all sounds to finish
}
```

# Features

* Low-latency audio playback
* Cross-platform audio playback (including wasm)
* Handle device changes or disconnects in real time
* Low CPU usage
* Minimal dependencies
* Minimal memory allocations
* No `panic!()` or `.unwrap()`, always propogate errors
* No unsafe code
* Simple API, while being customizable
* Optionally use [Symphonia](https://github.com/pdeljanov/Symphonia) to support most audio formats
* Feature to disable audio playback support, if you want to use kittyaudio purely as an audio library
* Commands to change volume, playback rate and position in the sound with easings

# Roadmap

Those features are not implemented yet.

* Effects (reverb, delay, eq, etc.)
* C API
* Audio streaming from disk
