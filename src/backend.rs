use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;

use crate::KaError;
use crate::Renderer;
use crate::RendererHandle;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::FromSample;
use cpal::SampleFormat;
use cpal::SizedSample;
use cpal::StreamConfig;

/// Specifies what device [`cpal`] should use.
///
/// For example, if you want [`cpal`] to use the default OS audio device,
/// use [`Device::Default`]. If you want select a device by name, use `Device::Name("device name".to_string())`.
///
/// Use [`device_names`] to get all device names available on the system. The
/// [`Device`] struct also has methods for finding a device by name and getting
/// the default device as a [`Device::Custom`].
#[derive(Default)]
pub enum Device {
    /// Use the default OS audio device.
    #[default]
    Default,
    /// Specify a device by name.
    Name(String),
    /// Use a [`cpal::Device`].
    Custom(cpal::Device),
}

impl Device {
    /// Finds a [`cpal`] audio output device ([`cpal::Device`]) by name.
    pub fn from_name(name: &str) -> Result<Self, KaError> {
        let host = cpal::default_host();
        Ok(Self::Custom(
            host.output_devices()?
                .find(|d| device_name(d) == name)
                .ok_or(KaError::NoOutputDevice)?,
        ))
    }

    /// Get the default device as [`Device::Custom`].
    pub fn default_device() -> Result<Self, KaError> {
        let host = cpal::default_host();
        Ok(Self::Custom(
            host.default_output_device()
                .ok_or(KaError::NoOutputDevice)?,
        ))
    }
}

/// Returns all device names available on the system.
pub fn device_names() -> Result<Vec<String>, KaError> {
    let host = cpal::default_host();
    Ok(host.output_devices()?.map(|d| device_name(&d)).collect())
}

#[inline]
fn default_device_and_config() -> Result<(cpal::Device, StreamConfig), KaError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or(KaError::NoOutputDevice)?;
    let config = device.default_output_config()?.config();
    Ok((device, config))
}

#[inline]
fn device_name(device: &cpal::Device) -> String {
    device
        .name()
        .unwrap_or_else(|_| "<unavailable>".to_string())
}

/// Wrapper around [`cpal`]'s stream settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamSettings {
    /// Amount of channels. If [`None`], [`cpal`] provides the default value.
    pub channels: Option<u16>,
    /// Audio framerate. If [`None`], [`cpal`] provides the default value.
    pub sample_rate: Option<u32>,
    /// Audio buffer size (in samples). If [`None`], [`cpal`] provides the default value.
    pub buffer_size: Option<u32>,
    /// Amount of channels. If [`None`], [`cpal`] provides the default value.
    pub sample_format: Option<SampleFormat>,
    /// Whether to check the stream for device changes/disconnections.
    pub check_stream: bool,
    /// Interval at which to check the stream for device changes/disconnections.
    pub check_stream_interval: Duration,
}

impl Default for StreamSettings {
    fn default() -> Self {
        Self {
            channels: None,
            sample_rate: None,
            buffer_size: None,
            sample_format: None,
            check_stream: true,
            check_stream_interval: Duration::from_millis(500),
        }
    }
}

/// A wrapper around [`cpal`]'s stream. The [`Backend`] will check for device
/// changes or disconnections, handle errors and manage the stream.
#[derive(Default)]
pub struct Backend {
    /// Stream error queue.
    pub error_queue: Arc<Mutex<Vec<cpal::StreamError>>>,
    /// The interval at which the stream should be checked.
    pub check_stream_interval: Duration,
    /// Whether the stream should be checked.
    pub check_stream: bool,
    /// Whether to stop the stream at the next stream check.
    // TODO: how can we apply this faster?
    stop_stream: bool,
}

impl Backend {
    /// Creates a new [`Backend`].
    #[inline]
    pub fn new() -> Self {
        Self {
            error_queue: Arc::new(Mutex::new(Vec::new())),
            check_stream_interval: Duration::from_millis(500),
            check_stream: true,
            stop_stream: false,
        }
    }

    /// Handle all errors in the error queue.
    #[inline]
    pub fn handle_errors(&mut self, err_fn: impl FnMut(cpal::StreamError)) {
        self.error_queue
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain(..)
            .for_each(err_fn)
    }

    /// Starts the audio thread.
    pub fn start_audio_thread<R>(
        &mut self,
        device: Device,
        settings: StreamSettings,
        renderer: RendererHandle<R>,
    ) -> Result<(), KaError>
    where
        R: Renderer,
    {
        // cpal will panic if no default host is present, we can't do anything
        // about that
        let host = cpal::default_host();

        // get output device
        let device = match device {
            Device::Default => host
                .default_output_device()
                .ok_or(KaError::NoOutputDevice)?,
            Device::Name(name) => host
                .output_devices()?
                .find(|d| device_name(d) == name)
                .ok_or(KaError::NoOutputDevice)?,
            Device::Custom(device) => device,
        };

        // get supported stream config
        let default_config = device.default_output_config()?;
        let sample_format = settings
            .sample_format
            .unwrap_or_else(|| default_config.sample_format());

        // create modified stream config (if `settings` has [`Some`] values)
        let config = StreamConfig {
            channels: settings
                .channels
                .unwrap_or_else(|| default_config.config().channels),
            sample_rate: settings
                .sample_rate
                .map(cpal::SampleRate)
                .unwrap_or_else(|| default_config.sample_rate()),
            buffer_size: settings
                .buffer_size
                .map(cpal::BufferSize::Fixed)
                .unwrap_or(cpal::BufferSize::Default),
        };

        // update backend settings
        self.check_stream = settings.check_stream;
        self.check_stream_interval = settings.check_stream_interval;

        // check if this is a custom device
        let custom_device =
            if let Ok((default_device, default_config)) = default_device_and_config() {
                device_name(&device) != device_name(&default_device)
                    || config.sample_rate != default_config.sample_rate
            } else {
                false
            };

        // start the stream for the requested sample format
        use SampleFormat::*;
        match sample_format {
            I8 => self.start_stream::<i8, R>(&device, &config, renderer, custom_device)?,
            I16 => self.start_stream::<i16, R>(&device, &config, renderer, custom_device)?,
            // I24 => self.start_stream::<I24, R>(&device, &conf, I24.into(), renderer,custom_device)?,
            I32 => self.start_stream::<i32, R>(&device, &config, renderer, custom_device)?,
            // I48 => self.start_stream::<I48, R>(&device, &conf, I48.into(), renderer,custom_device)?,
            I64 => self.start_stream::<i64, R>(&device, &config, renderer, custom_device)?,
            U8 => self.start_stream::<u8, R>(&device, &config, renderer, custom_device)?,
            U16 => self.start_stream::<u16, R>(&device, &config, renderer, custom_device)?,
            // U24 => self.start_stream::<U24, R>(&device, &conf, U24.into(), renderer,custom_device)?,
            U32 => self.start_stream::<u32, R>(&device, &config, renderer, custom_device)?,
            // U48 => self.start_stream::<U48, R>(&device, &conf, U48.into(), renderer,custom_device)?,
            U64 => self.start_stream::<u64, R>(&device, &config, renderer, custom_device)?,
            F32 => self.start_stream::<f32, R>(&device, &config, renderer, custom_device)?,
            F64 => self.start_stream::<f64, R>(&device, &config, renderer, custom_device)?,
            sample_format => return Err(KaError::UnsupportedSampleFormat(sample_format)),
        }

        Ok(())
    }

    /// Stop the audio thread at the next stream check.
    #[inline(always)]
    pub fn stop_stream(&mut self) {
        self.stop_stream = true;
    }

    /// Return true if the audio stream should be restarted.
    fn check_stream(
        &mut self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        custom_device: bool,
    ) -> bool {
        // check for device disconnection
        let error_queue = self.error_queue.clone();
        for err in error_queue
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain(..)
        {
            if matches!(err, cpal::StreamError::DeviceNotAvailable) {
                return true;
            }
        }

        // check for device changes
        // disabled on macos due to audio artifacts that occur while a device is
        // being queried while a stream is playing
        #[cfg(not(target_os = "macos"))]
        if !custom_device {
            if let Ok((default_device, default_config)) = default_device_and_config() {
                if device_name(device) != device_name(&default_device)
                    || config.sample_rate != default_config.sample_rate
                {
                    return true;
                }
            }
        }

        false
    }

    /// Start the [`cpal`] stream.
    fn start_stream<T, R>(
        &mut self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        renderer: RendererHandle<R>,
        custom_device: bool,
    ) -> Result<(), KaError>
    where
        T: SizedSample + FromSample<f32>,
        R: Renderer,
    {
        let channels = config.channels as usize; // number of channels
        let sample_rate = config.sample_rate.0; // sample rate
        let error_queue = self.error_queue.clone(); // stream error queue

        // create a clone of the renderer handle so we can move it inside the
        // stream closure
        let renderer_moved = renderer.clone();

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_exact_mut(channels) {
                    // mix next frame
                    let out = renderer_moved.guard().next_frame(sample_rate);

                    // write to buffer
                    if channels == 1 {
                        // mix both channels
                        frame[0] = T::from_sample((out.left + out.right) / 2.0);
                    } else {
                        frame[0] = T::from_sample(out.left);
                        frame[1] = T::from_sample(out.right);

                        // if there are more than 2 channels, send silence to them,
                        // otherwise we might leave some garbage in there
                        for channel in frame.iter_mut().skip(2) {
                            *channel = T::from_sample(0.);
                        }
                    }
                }
            },
            move |err| {
                // we got an error on stream, push it to the error queue
                error_queue
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(err)
            },
            None,
        )?;

        // start cpal's audio playback thread
        stream.play()?;

        // periodically check for device changes
        loop {
            std::thread::sleep(self.check_stream_interval);

            // check stream
            if self.check_stream && self.check_stream(device, config, custom_device) {
                drop(stream); // stop this stream so we can start a new one
                return self.start_audio_thread(
                    Device::Default,
                    StreamSettings::default(),
                    renderer,
                );
            }

            // see if we should stop the stream
            if self.stop_stream {
                self.stop_stream = false;
                drop(stream); // stop stream
                break;
            }
        }
        Ok(())
    }
}
