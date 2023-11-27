# 🐱 kittyaudio

kittyaudio is a Rust audio playback library focusing on simplicity, speed and low-latency audio playback.

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

# Goals

* Low-latency audio playback
* Cross-platform audio playback (including wasm)
* Handle device changes or disconnects in real time
* Low CPU usage
* Minimal dependencies
* Minimal memory allocations
* Streaming and in-memory audio playback
* No `panic!()` or `.unwrap()`, always propogate errors
* No unsafe code
* Simple API, while being customizable
* Optionally use [Symphonia](https://github.com/pdeljanov/Symphonia) to support most audio formats
