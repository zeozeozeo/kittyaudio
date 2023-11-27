use crate::resampler::Resampler;
use std::ops::AddAssign;
use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

#[cfg(feature = "use-symphonia")]
use {crate::KaError, std::io::Cursor};

#[macro_export]
#[cfg(feature = "use-symphonia")]
macro_rules! include_sound {
    ($path:expr) => {
        $crate::Sound::from_cursor(::std::io::Cursor::new(include_bytes!($path)))
    };
}

#[cfg(feature = "use-symphonia")]
use symphonia::core::{
    audio::Signal,
    audio::{AudioBuffer, AudioBufferRef},
    conv::{FromSample, IntoSample},
    io::MediaSource,
    sample::Sample,
};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Frame {
    pub left: f32,
    pub right: f32,
}

impl Frame {
    pub const ZERO: Self = Self {
        left: 0.0,
        right: 0.0,
    };

    #[inline]
    pub const fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    #[inline]
    pub const fn from_mono(value: f32) -> Self {
        Self::new(value, value)
    }
}

impl From<[f32; 2]> for Frame {
    fn from(lr: [f32; 2]) -> Self {
        Self::new(lr[0], lr[1])
    }
}

impl From<(f32, f32)> for Frame {
    fn from(lr: (f32, f32)) -> Self {
        Self::new(lr.0, lr.1)
    }
}

impl From<f32> for Frame {
    fn from(value: f32) -> Self {
        Self::from_mono(value)
    }
}

impl Add for Frame {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.left + rhs.left, self.right + rhs.right)
    }
}

impl AddAssign for Frame {
    fn add_assign(&mut self, rhs: Self) {
        self.left += rhs.left;
        self.right += rhs.right;
    }
}

impl Sub for Frame {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.left - rhs.left, self.right - rhs.right)
    }
}

impl SubAssign for Frame {
    fn sub_assign(&mut self, rhs: Self) {
        self.left -= rhs.left;
        self.right -= rhs.right;
    }
}

impl Mul<f32> for Frame {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.left * rhs, self.right * rhs)
    }
}

impl MulAssign<f32> for Frame {
    fn mul_assign(&mut self, rhs: f32) {
        self.left *= rhs;
        self.right *= rhs;
    }
}

impl Div<f32> for Frame {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.left / rhs, self.right / rhs)
    }
}

impl DivAssign<f32> for Frame {
    fn div_assign(&mut self, rhs: f32) {
        self.left /= rhs;
        self.right /= rhs;
    }
}

impl Neg for Frame {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.left, -self.right)
    }
}

/// Specifies how quickly the sound is played.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlaybackRate {
    /// The sound is playing at a speed factor of the original sample rate.
    ///
    /// For example, `PlaybackRate::Factor(4.0)` means that the sound is
    /// played 4 times as fast compared to `PlaybackRate::Factor(1.0)`
    Factor(f64),
    /// The speed of the sound is controlled in a way that the pitch is
    /// adjusted by the given number of semitones.
    Semitones(f64),
}

impl Default for PlaybackRate {
    fn default() -> Self {
        Self::Factor(1.0)
    }
}

impl PlaybackRate {
    /// Returns the playback rate as a factor of the original sample rate.
    #[inline] // float arithmetic is not allowed in const fns
    pub fn as_factor(self) -> f64 {
        match self {
            PlaybackRate::Factor(factor) => factor,
            PlaybackRate::Semitones(semitones) => 2.0f64.powf(semitones / 12.0),
        }
    }

    /// Returns the amount of semitones (pitch difference) this playback rate
    /// would result in.
    #[inline] // float arithmetic is not allowed in const fns
    pub fn as_semitones(self) -> f64 {
        match self {
            PlaybackRate::Factor(factor) => 12.0 * factor.log2(),
            PlaybackRate::Semitones(semitones) => semitones,
        }
    }
}

impl From<f64> for PlaybackRate {
    fn from(factor: f64) -> Self {
        Self::Factor(factor)
    }
}

/// Audio data stored in memory. This type can be cheaply cloned, as the
/// audio data is shared between all clones.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    /// Sample rate of the sound.
    sample_rate: u32,
    pub frames: Arc<[Frame]>,
    /// Whether the sound is paused.
    pub paused: bool,
    /// The current playback position in frames.
    index: usize,
    /// The resampler used to resample the audio data.
    resampler: Resampler,
    /// The current playback rate of the sound. See [`PlaybackRate`] for more
    /// details.
    playback_rate: PlaybackRate,
    /// Fractional position between samples. Always in the range of 0-1.
    fractional_position: f64,
    /// Current volume of the samples pushed to the resampler.
    volume: f32,
}

impl Default for Sound {
    fn default() -> Self {
        let mut sound = Self {
            sample_rate: 0,
            frames: Arc::new([]),
            paused: false,
            index: 0,
            resampler: Resampler::new(0),
            playback_rate: PlaybackRate::Factor(1.0),
            fractional_position: 0.0,
            volume: 1.0,
        };

        // fill the resampler with 3 audio frames so the playback starts
        // immediately (the resampler needs 4 samples to output any audio)
        for _ in 0..3 {
            sound.update_position();
        }

        sound
    }
}

/// Helper function to convert Symphonia's [`AudioBufferRef`] to a vector of [`Frame`]s.
#[cfg(feature = "use-symphonia")]
fn load_frames_from_buffer_ref(buffer: &AudioBufferRef) -> Result<Vec<Frame>, KaError> {
    match buffer {
        AudioBufferRef::U8(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::U16(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::U24(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::U32(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::S8(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::S16(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::S24(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::S32(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::F32(buffer) => load_frames_from_buffer(buffer),
        AudioBufferRef::F64(buffer) => load_frames_from_buffer(buffer),
    }
}

/// Convert an [`AudioBuffer`] into a [`Vec`] of [`Frame`]s.
#[cfg(feature = "use-symphonia")]
fn load_frames_from_buffer<S: Sample>(buffer: &AudioBuffer<S>) -> Result<Vec<Frame>, KaError>
where
    f32: FromSample<S>,
{
    let num_channels = buffer.spec().channels.count();
    match num_channels {
        1 => Ok(buffer
            .chan(0)
            .iter()
            .map(|sample| Frame::from_mono((*sample).into_sample()))
            .collect()),
        2 => Ok(buffer
            .chan(0)
            .iter()
            .zip(buffer.chan(1).iter())
            .map(|(left, right)| Frame::new((*left).into_sample(), (*right).into_sample()))
            .collect()),
        _ => Err(KaError::UnsupportedNumberOfChannels(num_channels as _)),
    }
}

impl Sound {
    /// Make a [`Sound`] from [`symphonia`]'s [`Box`]'ed [`MediaSource`].
    ///
    /// Required features: `use-symphonia`
    #[cfg(feature = "use-symphonia")]
    pub fn from_boxed_media_source(media_source: Box<dyn MediaSource>) -> Result<Self, KaError> {
        use std::io::ErrorKind::UnexpectedEof;
        use symphonia::core::codecs::DecoderOptions;
        use symphonia::core::errors::Error;
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        // create a media source stream from the provided media source
        let mss = MediaSourceStream::new(media_source, Default::default());

        // create a hint to help the format registry to guess what format
        // the media source is using, we'll let symphonia figure that out for us
        let hint = Hint::new();

        // use default options for reading and encoding
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // probe the media source for a format
        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        let mut format = probed.format;
        let Some(track) = format.default_track() else {
            return Err(KaError::NoTracksArePresent); // failed to get default track
        };

        // create a decoder for the track
        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts)?;

        // store the track identifier, we'll use it to filter packets
        let track_id = track.id;

        // get sample rate
        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or(KaError::UnknownSampleRate)?;

        let mut frames = Vec::new(); // audio data

        loop {
            // get the next packet from the format reader
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(Error::IoError(e)) => {
                    // if we reached eof, stop decoding
                    if e.kind() == UnexpectedEof {
                        break;
                    }
                    // ...otherwise return KaError
                    return Err(Error::IoError(e).into());
                }
                Err(e) => return Err(e.into()), // not io error
            };

            // if the packet does not belong to the selected track, skip it
            if packet.track_id() != track_id {
                continue;
            }

            // decode packet
            let buffer = decoder.decode(&packet)?;
            frames.append(&mut load_frames_from_buffer_ref(&buffer)?);
        }

        Ok(Self {
            sample_rate,
            frames: frames.into(),
            ..Default::default()
        })
    }

    /// Make a [`Sound`] from [`symphonia`]'s [`MediaSource`].
    ///
    /// Required features: `use-symphonia`
    #[cfg(feature = "use-symphonia")]
    #[inline]
    pub fn from_media_source(media_source: impl MediaSource + 'static) -> Result<Self, KaError> {
        Self::from_boxed_media_source(Box::new(media_source))
    }

    /// Make a [`Sound`] from a [`Cursor`] of bytes. Uses [`symphonia`] to decode audio.
    ///
    /// Required features: `use-symphonia`
    #[cfg(feature = "use-symphonia")]
    #[inline]
    pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
        cursor: Cursor<T>,
    ) -> Result<Self, KaError> {
        Self::from_media_source(cursor)
    }

    /// Make a [`Sound`] from a file path. Uses [`symphonia`] to decode audio.
    ///
    /// Required features: `use-symphonia`
    #[cfg(feature = "use-symphonia")]
    #[inline]
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, KaError> {
        Self::from_media_source(std::fs::File::open(path)?)
    }

    /// Make a [`Sound`] from a [`Vec`] of bytes ([`u8`]). Uses [`symphonia`] to decode audio.
    ///
    /// Required features: `use-symphonia`
    #[cfg(feature = "use-symphonia")]
    #[inline]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, KaError> {
        Self::from_cursor(Cursor::new(bytes))
    }

    /// Make a [`Sound`] from a slice of [`Frame`]s and a sample rate.
    #[inline]
    pub fn from_frames(sample_rate: u32, frames: &[Frame]) -> Self {
        Self {
            sample_rate,
            frames: frames.into(),
            ..Default::default()
        }
    }

    /// Return the sample rate of the sound.
    #[inline]
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Return the duration of the sound.
    ///
    /// Returns [`Duration`].
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.duration_seconds())
    }

    /// Return the duration of the sound in seconds.
    #[inline]
    pub fn duration_seconds(&self) -> f64 {
        self.frames.len() as f64 / self.sample_rate as f64
    }

    /// Push the current frame (pointed by `self.index`) to the resampler.
    pub fn push_frame_to_resampler(&mut self) {
        let frame_index = self.index; // copy to stack
        self.resampler.push_frame(
            // push silence if index is out of the range
            *self.frames.get(frame_index).unwrap_or(&Frame::ZERO) * self.volume,
            frame_index,
        );
    }

    /// Increment/decrement the position value in the sound, pushing the
    /// previous sound frame to the resampler.
    pub fn update_position(&mut self) {
        if self.paused {
            self.resampler.push_frame(Frame::ZERO, self.index);
        } else {
            self.push_frame_to_resampler();
            self.index += 1; // TODO: loops
        }
    }

    /// Return whether the sound has finished playback.
    #[inline]
    pub fn finished(&self) -> bool {
        self.index >= self.frames.len()
    }

    /// Render the next frame. If the sound has ended, return `Frame::ZERO`.
    #[inline]
    pub fn next_frame(&mut self, sample_rate: u32) -> Frame {
        if self.finished() {
            return Frame::ZERO;
        }

        // get resampled frame
        let frame = self.resampler.get(self.fractional_position as f32);

        // increment fractional position
        self.fractional_position +=
            (self.sample_rate as f64 / sample_rate as f64) * self.playback_rate.as_factor().abs();

        // step the corrent amount of samples forward/backward
        while self.fractional_position >= 1.0 {
            self.fractional_position -= 1.0;
            self.update_position();
        }

        frame
    }

    /// Reset the sound to the beginning.
    #[inline]
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Set the playback rate of the sound. See [`PlaybackRate`] for more
    /// details. Returns the previous playback rate.
    #[inline]
    pub fn set_playback_rate(&mut self, playback_rate: PlaybackRate) -> PlaybackRate {
        let prev_playback_rate = self.playback_rate;
        self.playback_rate = playback_rate;
        prev_playback_rate
    }

    /// Set the current volume. Return the previous volume value.
    #[inline]
    pub fn set_volume(&mut self, volume: f32) -> f32 {
        let prev_volume = self.volume;
        self.volume = volume;
        prev_volume
    }
}

/// Wraps a [`Sound`] so it can be returned to the user after `play`.
///
/// This type can be cheaply cloned, and it will share the same data.
#[derive(Debug, Clone)]
pub struct SoundHandle(Arc<Mutex<Sound>>);

impl SoundHandle {
    /// Create a new [`SoundHandle`] from a [`Sound`].
    #[inline]
    pub fn new(sound: Sound) -> Self {
        Self(Arc::new(Mutex::new(sound)))
    }

    /// Get a lock on the underlying [`Sound`].
    #[inline]
    pub fn guard(&self) -> MutexGuard<'_, Sound> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Return the sample rate of the sound.
    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.guard().sample_rate()
    }

    /// Return the duration of the sound.
    ///
    /// Returns [`Duration`].
    #[inline]
    pub fn duration(&self) -> Duration {
        self.guard().duration()
    }

    /// Return the duration of the sound in seconds.
    #[inline]
    pub fn duration_seconds(&self) -> f64 {
        self.guard().duration_seconds()
    }

    /// Return whether the sound has finished playback.
    #[inline]
    pub fn finished(&self) -> bool {
        self.guard().finished()
    }

    /// Reset the sound to the beginning.
    #[inline]
    pub fn reset(&self) {
        self.guard().reset();
    }

    /// Set the playback rate of the sound. See [`PlaybackRate`] for more
    /// details. Returns the previous playback rate.
    #[inline]
    pub fn set_playback_rate(&self, playback_rate: PlaybackRate) -> PlaybackRate {
        self.guard().set_playback_rate(playback_rate)
    }

    /// Return the current playback rate.
    #[inline]
    pub fn playback_rate(&self) -> PlaybackRate {
        self.guard().playback_rate
    }

    /// Return the current index in the source sound data.
    #[inline]
    pub fn index(&self) -> usize {
        self.guard().index
    }

    /// Return the current volume of the sound.
    #[inline]
    pub fn volume(&self) -> f32 {
        self.guard().volume
    }

    /// Set the current volume. Return the previous volume value.
    pub fn set_volume(&self, volume: f32) -> f32 {
        self.guard().set_volume(volume)
    }
}
