use crate::{Backend, Device, Frame, Sound, SoundHandle, StreamSettings};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Audio renderer.
pub struct Renderer {
    /// The streams' audio sample rate.
    pub sample_rate: u32,
    /// All playing sounds.
    pub sounds: Vec<SoundHandle>,
}

impl Renderer {
    /// Render the next audio frame.
    pub fn next_frame(&mut self) -> Frame {
        // mix samples from all playing sounds
        let mut out = Frame::ZERO;
        for sound in &mut self.sounds {
            out += sound.guard().next_frame(self.sample_rate);
        }

        // remove all sounds that finished playback
        self.sounds.retain(|s| !s.finished());

        out
    }
}

/// Wraps [`Renderer`] so it can be shared between threads.
#[derive(Clone)]
pub struct RendererHandle(Arc<Mutex<Renderer>>);

impl RendererHandle {
    /// Create a new renderer handle.
    pub fn new(renderer: Renderer) -> Self {
        Self(Arc::new(Mutex::new(renderer)))
    }

    /// Get a lock on the underlying renderer.
    #[inline(always)]
    pub fn guard(&self) -> MutexGuard<'_, Renderer> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Audio mixer. The mixing is done by the [`Renderer`] ([`RendererHandle`]),
/// and the audio playback is handled by the [`Backend`].
#[derive(Clone)]
pub struct Mixer {
    /// Handle to the underlying audio renderer.
    pub renderer: RendererHandle,
    /// Handle to the underlying audio backend.
    pub backend: Arc<Mutex<Backend>>,
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    /// Create a new audio mixer.
    pub fn new() -> Self {
        Self {
            // backend: Arc::new(Backend::new()),
            renderer: RendererHandle::new(Renderer {
                sample_rate: 44100,
                sounds: Vec::new(),
            }),
            backend: Arc::new(Mutex::new(Backend::new())),
        }
    }

    /// Get a lock on the underlying backend.
    #[inline(always)]
    pub fn backend(&self) -> MutexGuard<'_, Backend> {
        self.backend.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Play a [`Sound`].
    ///
    /// Note: Cloning a [`Sound`] *does not* take any extra memory, as [`Sound`]
    /// shares frame data with all clones.
    #[inline]
    pub fn play(&mut self, sound: Sound) -> SoundHandle {
        let handle = SoundHandle::new(sound);
        self.renderer.guard().sounds.push(handle.clone());
        handle
    }

    /// Handle stream errors.
    #[inline]
    pub fn handle_errors(&mut self, err_fn: impl FnMut(cpal::StreamError)) {
        self.backend().handle_errors(err_fn);
    }

    /// Start the audio thread with default backend settings.
    #[inline]
    pub fn init(&self) {
        self.init_ex(Device::Default, StreamSettings::default());
    }

    /// Start the audio thread with custom backend settings.
    ///
    /// * `device`: The audio device to use. Set to `Device::Default` for defaults.
    /// * `stream_config`: The audio stream configuration. Set to [`None`] for defaults.
    /// * `sample_format`: The audio sample format. Set to [`None`] for defaults.
    pub fn init_ex(&self, device: Device, settings: StreamSettings) {
        let backend = self.backend.clone();
        let renderer = self.renderer.clone();
        std::thread::spawn(move || {
            // TODO: handle errors from `start_audio_thread`
            let _ = backend
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .start_audio_thread(device, settings, renderer);
        });
    }

    /// Block the thread until all sounds are finished.
    pub fn wait(&self) {
        while !self.renderer.guard().sounds.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    /// Return whether all sounds are finished or not.
    #[inline]
    pub fn is_finished(&self) -> bool {
        // this vector is updated every audio frame to GC sounds that are not playing
        self.renderer.guard().sounds.is_empty()
    }
}
