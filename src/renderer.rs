use crate::{Frame, SoundHandle};
use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

/// The audio renderer trait. Can be used to make custom audio renderers.
pub trait Renderer: Clone + Send + 'static {
    /// Render the next audio frame. The backend provides the sample rate and
    /// expects the left and right channel values ([`Frame`]).
    ///
    /// Note: you can use a [`crate::Resampler`] to resample audio data.
    fn next_frame(&mut self, sample_rate: u32) -> Frame;

    /// This gets called when an audio buffer is done processing.
    #[cfg(feature = "cpal")]
    fn on_buffer<T>(&mut self, _buffer: &mut [T])
    where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
    }
}

/// Default audio renderer.
#[derive(Debug, Clone, Default)]
pub struct DefaultRenderer {
    /// All playing sounds.
    pub sounds: Vec<SoundHandle>,
    /// The last buffer size given by the [cpal] backend.
    pub last_buffer_size: usize,
}

impl DefaultRenderer {
    /// Start playing a sound. Accepts a type that can be converted into a
    /// [`SoundHandle`].
    #[inline]
    pub fn add_sound(&mut self, sound: impl Into<SoundHandle>) {
        self.sounds.push(sound.into());
    }

    /// Return whether the renderer has any playing sounds.
    pub fn has_sounds(&self) -> bool {
        !self.sounds.is_empty()
    }
}

impl Renderer for DefaultRenderer {
    fn next_frame(&mut self, sample_rate: u32) -> Frame {
        // mix samples from all playing sounds
        let mut out = Frame::ZERO;

        // remove all sounds that finished playback
        self.sounds.retain_mut(|sound| {
            let frame = sound.next_frame(sample_rate);
            if let Some(frame) = frame {
                out += frame;
                true
            } else {
                false
            }
        });

        out
    }

    #[cfg(feature = "cpal")]
    fn on_buffer<T>(&mut self, buffer: &mut [T])
    where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
        self.last_buffer_size = buffer.len();
    }
}

/// Wraps [`Renderer`] so it can be shared between threads.
#[derive(Clone)]
pub struct RendererHandle<R: Renderer>(Arc<Mutex<R>>);

impl From<DefaultRenderer> for RendererHandle<DefaultRenderer> {
    fn from(val: DefaultRenderer) -> Self {
        RendererHandle::new(val)
    }
}

impl<R: Renderer> RendererHandle<R> {
    /// Create a new renderer handle.
    pub fn new(renderer: R) -> Self {
        Self(Arc::new(Mutex::new(renderer)))
    }

    /// Get a lock on the underlying renderer.
    #[inline(always)]
    pub fn guard(&self) -> MutexGuard<'_, R> {
        self.0.lock()
    }
}
