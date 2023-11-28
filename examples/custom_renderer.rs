//! Example of creating a backend with a custom renderer.
use kittyaudio::{Backend, Device, Frame, Renderer, RendererHandle, StreamSettings};
use std::{thread, time::Duration};

#[derive(Clone)]
struct CustomRenderer {
    frame: usize,
}

impl Renderer for CustomRenderer {
    fn next_frame(&mut self, sample_rate: u32) -> Frame {
        self.frame += 1;
        let time = self.frame as f64 / sample_rate as f64;
        let value = (time.sin() * 1200.0).sin() * 0.5;
        Frame::from_mono(value as f32)
    }
}

fn main() {
    let renderer = RendererHandle::new(CustomRenderer { frame: 0 });

    thread::spawn(|| {
        let mut backend = Backend::new();
        backend
            .start_audio_thread(Device::Default, StreamSettings::default(), renderer)
            .expect("failed to start audio thread");
    });

    thread::sleep(Duration::from_secs(30));
}
