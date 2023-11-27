use crate::Frame;

/// Stores an audio frame and the frame index of that frame.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct ResamplerFrame {
    /// An audio frame.
    frame: Frame,
    /// The frame index at the time that this frame was pushed to the
    /// resampler.
    index: usize,
}

/// Resamples audio from one sample rate to another.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Resampler {
    /// Recent 4 frames with their frame index.
    /// Frame order: previous, current, next, next next.
    frames: [ResamplerFrame; 4],
}

/// This is the 4-point, 3rd-order Hermite interpolation x-form algorithm from
/// "Polynomial Interpolators for High-Quality Resampling of Oversampled Audio"
/// by Olli Niemitalo, p. 43:
/// http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
#[inline] // can't be const because of [`Frame`]'s Add/Sub/Mul impls
pub fn interpolate_frame(
    previous: Frame,
    current: Frame,
    next: Frame,
    next_next: Frame,
    fraction: f32,
) -> Frame {
    let c0 = current;
    let c1 = (next - previous) * 0.5;
    let c2 = previous - current * 2.5 + next * 2.0 - next_next * 0.5;
    let c3 = (next_next - previous) * 0.5 + (current - next) * 1.5;
    ((c3 * fraction + c2) * fraction + c1) * fraction + c0
}

impl Resampler {
    /// Create a new [`Resampler`].
    #[inline]
    pub const fn new(starting_index: usize) -> Self {
        Self {
            frames: [ResamplerFrame {
                frame: Frame::ZERO,
                index: starting_index,
            }; 4],
        }
    }

    /// Push a new frame to the resampler.
    #[inline]
    pub fn push_frame(&mut self, frame: Frame, frame_index: usize) {
        // move all samples to the right except the last one
        for i in 0..self.frames.len() - 1 {
            self.frames[i] = self.frames[i + 1];
        }
        // set this as the last sample
        // sample order: previous, current, next, next next
        self.frames[self.frames.len() - 1] = ResamplerFrame {
            frame,
            index: frame_index,
        };
    }

    /// Get an interpolated frame from a resampler at a fractional value.
    #[inline]
    pub fn get(&self, fraction: f32) -> Frame {
        interpolate_frame(
            self.frames[0].frame,
            self.frames[1].frame,
            self.frames[2].frame,
            self.frames[3].frame,
            fraction,
        )
    }

    /// Return the index of the frame in the source sound that is currently
    /// playing in the audio stream.
    ///
    /// This is not the same as the most recently pushed frame, as the stream
    /// mainly recieves an interpolated frame between `self.frames[1]` and
    /// `self.frames[2]`. `self.frames[0]` and `self.frames[3]` are used for
    /// the frame interpolation algorithm (see [`interpolate_frame`]).
    #[inline]
    pub const fn current_frame_index(&self) -> usize {
        self.frames[1].index
    }

    /// Return whether the resampler is outputting silence.
    #[inline]
    pub fn outputting_silence(&self) -> bool {
        self.frames
            .iter()
            .all(|ResamplerFrame { frame, .. }| *frame == Frame::ZERO)
    }
}
