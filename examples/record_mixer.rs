//! Demonstrates using a custom mixer for recording audio.
//! This example does NOT require the `playback` feature.

use std::time::Instant;

use kittyaudio::{
    include_sound, DefaultRenderer, Frame, Renderer, RendererHandle, Sound, SoundHandle,
};

struct RecordMixer {
    renderer: RendererHandle<DefaultRenderer>,
}

impl RecordMixer {
    /// Play a sound.
    fn play(&mut self, sound: Sound) -> SoundHandle {
        let handle = SoundHandle::new(sound);
        self.renderer.guard().add_sound(handle.clone());
        handle
    }

    /// Fill a given buffer with audio data.
    fn record_frames(&mut self, sample_rate: u32, frames: &mut [Frame]) {
        for frame in frames {
            // DefaultRenderer will handle resampling for us
            *frame = self.renderer.guard().next_frame(sample_rate);
        }
    }
}

fn main() {
    println!("loading sound...");
    let sound = include_sound!("../assets/drozerix_-_crush.ogg").unwrap();
    let mut mixer = RecordMixer {
        renderer: DefaultRenderer::default().into(),
    };

    mixer.play(sound);

    println!("rendering audio...");
    let start = Instant::now();

    // make a buffer where we will store the samples
    let mut buffer = [Frame::default(); 4096]; // 32 kib

    while mixer.renderer.guard().has_sounds() {
        mixer.record_frames(44100, &mut buffer);

        // we recorded 4096 audio samples in an audio buffer, do something with them...
    }

    println!("rendered in {:?}", start.elapsed());
}
