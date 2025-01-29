use crate::{DefaultRenderer, Frame, Renderer, RendererHandle, SoundHandle};

#[allow(unused_imports)] // for comments
use crate::Sound;

#[cfg(feature = "cpal")]
use crate::{Backend, Device, StreamSettings};

use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

/// Audio mixer. The mixing is done by the [`Renderer`] ([`RendererHandle`]),
/// and the audio playback is handled by the [`Backend`].
#[derive(Clone)]
pub struct Mixer {
    /// Handle to the default audio renderer.
    pub renderer: RendererHandle<DefaultRenderer>,
    /// Handle to the underlying audio backend.
    #[cfg(feature = "cpal")]
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
            renderer: DefaultRenderer::default().into(),
            #[cfg(feature = "cpal")]
            backend: Arc::new(Mutex::new(Backend::new())),
        }
    }

    /// Get a lock on the underlying backend.
    #[cfg(feature = "cpal")]
    #[inline(always)]
    pub fn backend(&self) -> MutexGuard<'_, Backend> {
        self.backend.lock()
    }

    /// Play a [`Sound`].
    ///
    /// Note: Cloning a [`Sound`] *does not* take any extra memory, as [`Sound`]
    /// shares frame data with all clones.
    #[inline]
    pub fn play(&mut self, sound: impl Into<SoundHandle>) -> SoundHandle {
        let handle = sound.into();
        self.renderer.guard().add_sound(handle.clone());
        handle
    }

    /// Handle stream errors.
    #[inline]
    #[cfg(feature = "cpal")]
    pub fn handle_errors(&mut self, err_fn: impl FnMut(cpal::StreamError)) {
        self.backend().handle_errors(err_fn);
    }

    /// Start the audio thread with default backend settings.
    #[inline]
    #[cfg(feature = "cpal")]
    pub fn init(&self) {
        self.init_ex(Device::Default, StreamSettings::default());
    }

    /// Start the audio thread with custom backend settings.
    ///
    /// * `device`: The audio device to use. Set to `Device::Default` for defaults.
    /// * `stream_config`: The audio stream configuration. Set to [`None`] for defaults.
    /// * `sample_format`: The audio sample format. Set to [`None`] for defaults.
    #[cfg(feature = "cpal")]
    pub fn init_ex(&self, device: Device, settings: StreamSettings) {
        let backend = self.backend.clone();
        let renderer = self.renderer.clone();
        std::thread::spawn(move || {
            // TODO: handle errors from `start_audio_thread`
            let _ = backend
                .lock()
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
        !self.renderer.guard().has_sounds()
    }

    /// Render the next audio frame. See [`DefaultRenderer`] for details.
    #[inline]
    pub fn next_frame(&self, sample_rate: u32) -> Frame {
        self.renderer.guard().next_frame(sample_rate)
    }
}

/// A mixer for recording audio.
///
/// This mixer does not play the audio, only records it. See [`Mixer`] for a
/// mixer that supports audio playback.
pub struct RecordMixer {
    /// A handle to the default audio renderer.
    pub renderer: RendererHandle<DefaultRenderer>,
}

impl Default for RecordMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordMixer {
    /// Create a new audio recording mixer.
    pub fn new() -> Self {
        Self {
            renderer: DefaultRenderer::default().into(),
        }
    }

    /// Play a [`Sound`] in the recording mixer. The samples of the sound are
    /// only processed when `fill_buffer` is called.
    ///
    /// Note: Cloning a [`Sound`] *does not* take any extra memory, as [`Sound`]
    /// shares frame data with all clones.
    #[inline]
    pub fn play(&self, sound: impl Into<SoundHandle>) -> SoundHandle {
        let handle: SoundHandle = sound.into();
        self.renderer.guard().add_sound(handle.clone());
        handle
    }

    /// Return whether all sounds are finished or not.
    #[inline]
    pub fn is_finished(&self) -> bool {
        !self.renderer.guard().has_sounds()
    }

    /// Fill the given buffer with audio samples. When the buffer is processed,
    /// no other samples are rendered before the next call to this function.
    pub fn fill_buffer(&self, sample_rate: u32, frames: &mut [Frame]) {
        let mut renderer = self.renderer.guard(); // acquire lock for this entire function
        for frame in frames {
            *frame = renderer.next_frame(sample_rate);
        }
    }

    /// Render the next audio frame. See [`DefaultRenderer`] for details.
    #[inline]
    pub fn next_frame(&self, sample_rate: u32) -> Frame {
        self.renderer.guard().next_frame(sample_rate)
    }
}
