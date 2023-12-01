use crate::{lerp_f64, Change, Command, Parameter, Resampler, Tweenable};
use parking_lot::{Mutex, MutexGuard};
use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::ops::{AddAssign, RangeInclusive};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "symphonia")]
use {crate::KaError, std::io::Cursor};

/// Includes a sound in the executable. The `symphonia` feature must be
/// enabled for this macro to exist.
///
/// This is a shorthand for `Sound::from_cursor(Cursor::new(include_bytes!(path)))`.
#[macro_export]
#[cfg(feature = "symphonia")]
macro_rules! include_sound {
    ($path:expr) => {
        $crate::Sound::from_cursor(::std::io::Cursor::new(include_bytes!($path)))
    };
}

#[cfg(feature = "symphonia")]
use symphonia::core::{
    audio::Signal,
    audio::{AudioBuffer, AudioBufferRef},
    conv::{FromSample, IntoSample},
    io::MediaSource,
    sample::Sample,
};

/// Represents an audio sample. Stores a left and right channel.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
    /// Left channel value. Float.
    pub left: f32,
    /// Right channel value. Float.
    pub right: f32,
}

impl Frame {
    /// A frame with all channels set to 0.0.
    pub const ZERO: Self = Self {
        left: 0.0,
        right: 0.0,
    };

    /// Create a new audio frame from left and right values.
    #[inline]
    pub const fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    /// Create a new audio frame from a single value.
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            Self::Factor(factor) => factor,
            Self::Semitones(semitones) => (semitones / 12.0).exp2(),
        }
    }

    /// Returns the amount of semitones (pitch difference) this playback rate
    /// would result in.
    #[inline] // float arithmetic is not allowed in const fns
    pub fn as_semitones(self) -> f64 {
        match self {
            Self::Factor(factor) => 12.0 * factor.log2(),
            Self::Semitones(semitones) => semitones,
        }
    }

    /// Reverse the playback rate so the sound plays backwards.
    pub fn reverse(self) -> Self {
        match self {
            Self::Factor(factor) => Self::Factor(-factor),
            Self::Semitones(semitones) => Self::Semitones(-semitones),
        }
    }
}

impl From<f64> for PlaybackRate {
    fn from(factor: f64) -> Self {
        Self::Factor(factor)
    }
}

impl Tweenable for PlaybackRate {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        match a {
            Self::Factor(factor) => Self::Factor(lerp_f64(factor, b.as_factor(), t as f64)),
            Self::Semitones(semitones) => {
                Self::Semitones(lerp_f64(semitones, b.as_semitones(), t as f64))
            }
        }
    }
}

/// Specifies a loop region.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct LoopPoints {
    /// Start of the loop as an index in the source data.
    pub start: usize,
    /// End of the loop as an index in the source data.
    pub end: usize,
}

impl LoopPoints {
    /// No loop.
    const NO_LOOP: Self = Self {
        start: 0,
        end: usize::MAX,
    };

    /// Make [`LoopPoints`] from an index range.
    #[inline]
    pub const fn from_range(range: RangeInclusive<usize>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
        }
    }

    /// Make [`LoopPoints`] from a seconds range.
    #[inline]
    pub fn from_range_secs(range: RangeInclusive<f64>, sample_rate: u32) -> Self {
        Self {
            start: (range.start() * sample_rate as f64) as usize,
            end: (range.end() * sample_rate as f64) as usize,
        }
    }

    /// Get start value in seconds.
    #[inline]
    fn start_secs(&self, sample_rate: u32) -> f64 {
        self.start as f64 / sample_rate as f64
    }

    /// Get end value in seconds.
    #[inline]
    fn end_secs(&self, sample_rate: u32) -> f64 {
        self.end as f64 / sample_rate as f64
    }
}

impl Tweenable for LoopPoints {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        Self {
            start: lerp_f64(a.start as f64, b.start as f64, t as f64) as usize,
            end: lerp_f64(a.end as f64, b.end as f64, t as f64) as usize,
        }
    }
}

/// Audio data stored in memory. This type can be cheaply cloned, as the
/// audio data is shared between all clones.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    /// Sample rate of the sound.
    sample_rate: u32,
    /// Audio data. Not mutable. Shared between all clones.
    pub frames: Arc<[Frame]>,
    /// Whether the sound is paused.
    pub paused: bool,
    /// The current playback position in frames.
    index: Parameter<usize>,
    /// The resampler used to resample the audio data.
    resampler: Resampler,
    /// The current playback rate of the sound. See [`PlaybackRate`] for more
    /// details.
    playback_rate: Parameter<PlaybackRate>,
    /// Fractional position between samples. Always in the range of 0-1.
    fractional_position: f64,
    /// Current volume of the samples pushed to the resampler.
    volume: Parameter<f32>,
    /// All unfinished commands.
    commands: Vec<Command>,
    /// Current two loop points.
    loop_points: Parameter<LoopPoints>,
    /// Whether looping is enabled.
    pub loop_enabled: bool,
}

impl Default for Sound {
    fn default() -> Self {
        let mut sound = Self {
            sample_rate: 0,
            frames: Arc::new([]),
            paused: false,
            index: Parameter::new(0),
            resampler: Resampler::new(0),
            playback_rate: Parameter::new(PlaybackRate::Factor(1.0)),
            fractional_position: 0.0,
            volume: Parameter::new(1.0),
            commands: vec![],
            loop_points: Parameter::new(LoopPoints::NO_LOOP),
            loop_enabled: false,
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
#[cfg(feature = "symphonia")]
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
#[cfg(feature = "symphonia")]
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
        _ => Err(KaError::UnsupportedNumberOfChannels(num_channels as u32)),
    }
}

impl Sound {
    /// Make a [`Sound`] from [`symphonia`]'s [`Box`]'ed [`MediaSource`].
    ///
    /// Required features: `symphonia`
    #[cfg(feature = "symphonia")]
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
        let track = format.default_track().ok_or(KaError::NoTracksArePresent)?;

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
    /// Required features: `symphonia`
    #[cfg(feature = "symphonia")]
    #[inline]
    pub fn from_media_source(media_source: impl MediaSource + 'static) -> Result<Self, KaError> {
        Self::from_boxed_media_source(Box::new(media_source))
    }

    /// Make a [`Sound`] from a [`Cursor`] of bytes. Uses [`symphonia`] to decode audio.
    ///
    /// Required features: `symphonia`
    #[cfg(feature = "symphonia")]
    #[inline]
    pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
        cursor: Cursor<T>,
    ) -> Result<Self, KaError> {
        Self::from_media_source(cursor)
    }

    /// Make a [`Sound`] from a file path. Uses [`symphonia`] to decode audio.
    ///
    /// Required features: `symphonia`
    #[cfg(feature = "symphonia")]
    #[inline]
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, KaError> {
        Self::from_media_source(std::fs::File::open(path)?)
    }

    /// Make a [`Sound`] from a [`Vec`] of bytes ([`u8`]). Uses [`symphonia`] to decode audio.
    ///
    /// Required features: `symphonia`
    #[cfg(feature = "symphonia")]
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
        let frame_index = self.index.value;
        self.resampler.push_frame(
            // push silence if index is out of the range
            *self.frames.get(frame_index).unwrap_or(&Frame::ZERO) * self.volume.value,
            frame_index,
        );
    }

    /// Return whether the sound is playing backward.
    #[inline]
    pub fn is_playing_backwards(&mut self) -> bool {
        self.playback_rate.value.as_factor().is_sign_negative()
    }

    /// Increment/decrement the position value in the sound, pushing the
    /// previous sound frame to the resampler.
    pub fn update_position(&mut self) {
        if self.paused {
            self.resampler.push_frame(Frame::ZERO, self.index.value);
        } else {
            self.push_frame_to_resampler();

            // increment/decrement index
            if self.is_playing_backwards() {
                self.index.value -= 1;
            } else {
                self.index.value += 1
            }
        }
    }

    /// Return whether the sound has finished playback.
    #[inline]
    pub fn finished(&self) -> bool {
        self.index.value >= self.frames.len()
    }

    /// Render the next frame. If the sound has ended, return `Frame::ZERO`.
    #[inline]
    pub fn next_frame(&mut self, sample_rate: u32) -> Frame {
        if self.finished() {
            return Frame::ZERO;
        }

        if self.loop_enabled {
            self.update_loop(self.loop_points.value.start, self.loop_points.value.end);
        }

        // update commands
        if !self.commands.is_empty() {
            self.update_commands(1.0 / sample_rate as f64);
        }

        // get resampled frame
        let frame = self.resampler.get(self.fractional_position as f32);

        // increment fractional position
        self.fractional_position += (self.sample_rate as f64 / sample_rate as f64)
            * self.playback_rate.value.as_factor().abs();

        // step the corrent amount of samples forward/backward
        while self.fractional_position >= 1.0 {
            self.fractional_position -= 1.0;
            self.update_position();
        }

        frame
    }

    fn update_loop(&mut self, start: usize, end: usize) {
        let index = self.index.value;
        if self.is_playing_backwards() {
            if index <= start {
                self.seek_to_index(end);
            }
        } else if index >= end {
            self.seek_to_index(start);
        }
    }

    /// Reset the sound to the beginning.
    #[inline]
    pub fn reset(&mut self) {
        self.seek_to_index(0);
    }

    /// Set the playback rate of the sound. See [`PlaybackRate`] for more
    /// details. Returns the previous playback rate.
    #[inline]
    pub fn set_playback_rate(&mut self, playback_rate: PlaybackRate) -> PlaybackRate {
        let prev_playback_rate = self.playback_rate.value;
        self.playback_rate.start_tween(playback_rate);
        prev_playback_rate
    }

    /// Return the current playback rate value. Can be modified with commands.
    #[inline]
    pub fn playback_rate(&self) -> PlaybackRate {
        self.playback_rate.value
    }

    /// Return the current base playback rate value. Can't be modified with commands.
    #[inline]
    pub fn base_playback_rate(&self) -> PlaybackRate {
        self.playback_rate.base_value
    }

    /// Set the current volume. Return the previous volume value.
    #[inline]
    pub fn set_volume(&mut self, volume: f32) -> f32 {
        let prev_volume = self.volume.value;
        self.volume.start_tween(volume);
        prev_volume
    }

    /// Return the current volume value. Can be modified with commands.
    #[inline]
    pub fn volume(&self) -> f32 {
        self.volume.value
    }

    /// Return the current base volume value. Can't be modified with commands.
    #[inline]
    pub fn base_volume(&self) -> f32 {
        self.volume.base_value
    }

    /// Seek to an index in the source data.
    #[inline]
    pub fn seek_to_index(&mut self, index: usize) {
        self.index.start_tween(index);

        // if the sound is playing, push this frame to the resampler so it
        // doesn't get skipped
        if !self.paused {
            self.push_frame_to_resampler();
        }
    }

    /// Seek to the end of the sound.
    #[inline]
    pub fn seek_to_end(&mut self) {
        self.seek_to_index(self.frames.len().saturating_sub(1));
    }

    /// Seek by a specified amount of seconds.
    #[inline]
    pub fn seek_by(&mut self, seconds: f64) {
        let cur_position = self.index.value as f64 / self.sample_rate as f64;
        let position = cur_position + seconds;
        let index = (position * self.sample_rate as f64) as usize;
        self.seek_to_index(index);
    }

    /// Seek to a specified position in seconds.
    #[inline]
    pub fn seek_to(&mut self, seconds: f64) {
        let index = (seconds * self.sample_rate as f64) as usize;
        self.seek_to_index(index);
    }

    /// Reverse the playback rate so the sound plays backwards.
    #[inline]
    pub fn reverse(&mut self) {
        self.playback_rate
            .start_tween(self.playback_rate.value.reverse())
    }

    /// Add a command to the sound. See [`Command`] for more details.
    #[inline]
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command)
    }

    fn update_commands(&mut self, dt: f64) {
        self.commands.retain_mut(|command| {
            if command.start_after <= 0.0 {
                // compute value with easing
                // start_after will be negative, and it counts the amount of time
                // the sound has been running for
                let t = command.value((-command.start_after / command.duration) as f32);

                // apply change
                match &command.change {
                    Change::Volume(vol) => self.volume.update(*vol, t),
                    Change::Index(index) => {
                        self.index.update(*index, t);
                        // TODO: push frame to resampler
                    }
                    Change::Position(position) => {
                        let index = position * self.sample_rate as f64;
                        self.index.update(index as usize, t);
                        // TODO: push frame to resampler
                    }
                    Change::Pause(pause) => {
                        if t >= 0.5 {
                            self.paused = *pause;
                        }
                    }
                    Change::PlaybackRate(rate) => self.playback_rate.update(*rate, t),
                    Change::LoopSeconds(range) => self.loop_points.update(
                        LoopPoints::from_range_secs(range.clone(), self.sample_rate),
                        t,
                    ),
                    Change::LoopIndex(range) => self
                        .loop_points
                        .update(LoopPoints::from_range(range.clone()), t),
                }
            }

            // if start_after is negative, it measures the elapsed time the command
            // has been running
            command.start_after -= dt;

            // if the command has finished, stop the tween
            let is_running = -command.start_after < command.duration;
            if !is_running {
                match command.change {
                    Change::Volume(_) => self.volume.stop_tween(),
                    Change::Index(_) => self.index.stop_tween(),
                    Change::Position(_) => self.index.stop_tween(),
                    Change::Pause(_) => (),
                    Change::PlaybackRate(_) => self.playback_rate.stop_tween(),
                    Change::LoopSeconds(_) | Change::LoopIndex(_) => self.loop_points.stop_tween(),
                }
            }
            is_running // only keep commands that are running
        });
    }

    /// Set the loop points as an index in the source data.
    #[inline]
    pub fn set_loop_index(&mut self, loop_region: RangeInclusive<usize>) {
        self.loop_points
            .start_tween(LoopPoints::from_range(loop_region));
    }

    /// Set the current loop state (enabled/disabled). Return the previous loop state.
    #[inline]
    pub fn set_loop_enabled(&mut self, enabled: bool) -> bool {
        let prev_enabled = self.loop_enabled;
        self.loop_enabled = enabled;
        prev_enabled
    }

    /// Set the loop points as a position in seconds.
    #[inline]
    pub fn set_loop(&mut self, loop_region: RangeInclusive<f64>) {
        self.loop_points =
            Parameter::new(LoopPoints::from_range_secs(loop_region, self.sample_rate));
    }

    /// Return the starting point of the loop as an index in the source data.
    #[inline]
    pub fn loop_start(&self) -> usize {
        self.loop_points.value.start
    }

    /// Return the ending point of the loop as an index in the source data.
    #[inline]
    pub fn loop_end(&self) -> usize {
        self.loop_points.value.end
    }

    /// Return the starting point of the loop as seconds.
    #[inline]
    pub fn loop_start_secs(&self) -> f64 {
        self.loop_points.value.start_secs(self.sample_rate)
    }

    /// Return the ending point of the loop as seconds.
    #[inline]
    pub fn loop_end_secs(&self) -> f64 {
        self.loop_points.value.end_secs(self.sample_rate)
    }

    /// Return the current index in the source sound data. Can be modified with commands.
    #[inline]
    pub fn index(&self) -> usize {
        self.index.value
    }

    /// Return the current index in the source sound data. Cannot be modified with commands.
    #[inline]
    pub fn base_index(&self) -> usize {
        self.index.base_value
    }

    /// Return whether the sound is currently outputting silence.
    #[inline]
    pub fn outputting_silence(&self) -> bool {
        self.resampler.outputting_silence()
    }
}

/// Wraps a [`Sound`] so it can be returned to the user after `play`.
///
/// This type can be cheaply cloned, and it will share the same data.
#[derive(Debug, Clone)]
pub struct SoundHandle(Arc<Mutex<Sound>>);

impl From<Sound> for SoundHandle {
    fn from(sound: Sound) -> Self {
        Self::new(sound)
    }
}

// TODO: can we generate this with a macro?
impl SoundHandle {
    /// Create a new [`SoundHandle`] from a [`Sound`].
    #[inline]
    pub fn new(sound: impl Into<Sound>) -> Self {
        Self(Arc::new(Mutex::new(sound.into())))
    }

    /// Get a lock on the underlying [`Sound`].
    #[inline]
    pub fn guard(&self) -> MutexGuard<'_, Sound> {
        self.0.lock()
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
    /// Push the current frame (pointed by `self.index()`) to the resampler.
    #[inline]
    pub fn push_frame_to_resampler(&self) {
        self.guard().push_frame_to_resampler()
    }
    /// Return whether the sound is playing backward.
    #[inline]
    pub fn is_playing_backwards(&self) -> bool {
        self.guard().is_playing_backwards()
    }
    /// Increment/decrement the position value in the sound,
    /// pushing the previous sound frame to the resampler.
    #[inline]
    pub fn update_position(&self) {
        self.guard().update_position()
    }
    /// Return whether the sound has finished playback.
    #[inline]
    pub fn finished(&self) -> bool {
        self.guard().finished()
    }
    /// Render the next frame. If the sound has ended, return `Frame::ZERO`.
    #[inline]
    pub fn next_frame(&self, sample_rate: u32) -> Frame {
        self.guard().next_frame(sample_rate)
    }
    /// Reset the sound to the beginning.
    #[inline]
    pub fn reset(&self) {
        self.guard().reset()
    }
    /// Set the playback rate of the sound. See [PlaybackRate] for more details. Returns the previous playback rate.
    #[inline]
    pub fn set_playback_rate(&self, playback_rate: PlaybackRate) -> PlaybackRate {
        self.guard().set_playback_rate(playback_rate)
    }
    /// Return the current playback rate value. Can be modified with commands.
    #[inline]
    pub fn playback_rate(&self) -> PlaybackRate {
        self.guard().playback_rate()
    }
    /// Return the current base playback rate value. Can't be modified with commands.
    #[inline]
    pub fn base_playback_rate(&self) -> PlaybackRate {
        self.guard().base_playback_rate()
    }
    /// Set the current volume. Return the previous volume value.
    #[inline]
    pub fn set_volume(&self, volume: f32) -> f32 {
        self.guard().set_volume(volume)
    }
    /// Return the current volume value. Can be modified with commands.
    #[inline]
    pub fn volume(&self) -> f32 {
        self.guard().volume()
    }
    /// Return the current base volume value. Can't be modified with commands.
    #[inline]
    pub fn base_volume(&self) -> f32 {
        self.guard().base_volume()
    }
    /// Seek to an index in the source data.
    #[inline]
    pub fn seek_to_index(&self, index: usize) {
        self.guard().seek_to_index(index)
    }
    /// Seek to the end of the sound.
    #[inline]
    pub fn seek_to_end(&self) {
        self.guard().seek_to_end()
    }
    /// Seek by a specified amount of seconds.
    #[inline]
    pub fn seek_by(&self, seconds: f64) {
        self.guard().seek_by(seconds)
    }
    /// Seek to a specified position in seconds.
    #[inline]
    pub fn seek_to(&self, seconds: f64) {
        self.guard().seek_to(seconds)
    }
    /// Reverse the playback rate so the sound plays backwards.
    #[inline]
    pub fn reverse(&self) {
        self.guard().reverse()
    }
    /// Add a command to the sound. See [`Command`] for more details.
    #[inline]
    pub fn add_command(&self, command: Command) {
        self.guard().add_command(command)
    }
    /// Set the loop points as an index in the source data.
    #[inline]
    pub fn set_loop_index(&self, loop_region: RangeInclusive<usize>) {
        self.guard().set_loop_index(loop_region)
    }
    /// Set the current loop state (enabled/disabled). Return the previous loop state.
    #[inline]
    pub fn set_loop_enabled(&self, enabled: bool) -> bool {
        self.guard().set_loop_enabled(enabled)
    }
    /// Return the current loop state (enabled/disabled).
    #[inline]
    pub fn loop_enabled(&self) -> bool {
        self.guard().loop_enabled
    }
    /// Set the loop points as a position in seconds.
    #[inline]
    pub fn set_loop(&self, loop_region: RangeInclusive<f64>) {
        self.guard().set_loop(loop_region)
    }
    /// Return the starting point of the loop as an index in the source data.
    #[inline]
    pub fn loop_start(&self) -> usize {
        self.guard().loop_start()
    }
    /// Return the ending point of the loop as an index in the source data.
    #[inline]
    pub fn loop_end(&self) -> usize {
        self.guard().loop_end()
    }
    /// Return the starting point of the loop as seconds.
    #[inline]
    pub fn loop_start_secs(&self) -> f64 {
        self.guard().loop_start_secs()
    }
    /// Return the ending point of the loop as seconds.
    #[inline]
    pub fn loop_end_secs(&self) -> f64 {
        self.guard().loop_end_secs()
    }
    /// Return the current index in the source sound data. Can be modified with commands.
    #[inline]
    pub fn index(&self) -> usize {
        self.guard().index()
    }
    /// Return the current index in the source sound data. Cannot be modified with commands.
    #[inline]
    pub fn base_index(&self) -> usize {
        self.guard().base_index()
    }
    /// Return whether the sound is currently outputting silence.
    #[inline]
    pub fn outputting_silence(&self) -> bool {
        self.guard().outputting_silence()
    }
}
