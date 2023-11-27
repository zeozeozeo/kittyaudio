use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;

use crate::KaError;
use crate::RendererHandle;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::FromSample;
use cpal::SampleFormat;
use cpal::SizedSample;
use cpal::StreamConfig;

#[derive(Default)]
pub enum Device {
    #[default]
    Default,
    Name(String),
    Custom(cpal::Device),
}

/// Returns all device names available on the system.
pub fn device_names() -> Result<Vec<String>, KaError> {
    let host = cpal::default_host();
    Ok(host
        .output_devices()?
        .map(|d| d.name().unwrap_or_default())
        .collect())
}

/// Finds a [`cpal`] audio output device ([`cpal::Device`]) by name.
pub fn get_device_by_name(name: &str) -> Result<cpal::Device, KaError> {
    let host = cpal::default_host();
    host.output_devices()?
        .find(|d| d.name().unwrap_or_default() == name)
        .ok_or(KaError::NoOutputDevice)
}

/// Returns the default cpal audio output device ([`cpal::Device`]).
pub fn get_default_device() -> Result<cpal::Device, KaError> {
    let host = cpal::default_host();
    host.default_output_device().ok_or(KaError::NoOutputDevice)
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

#[derive(Default)]
pub struct Backend {
    pub error_queue: Arc<Mutex<Vec<cpal::StreamError>>>,
    pub check_stream_interval: Duration,
    pub check_stream: bool,
    stop_stream: bool,
}

impl Backend {
    #[inline]
    pub fn new() -> Self {
        Self {
            error_queue: Arc::new(Mutex::new(Vec::new())),
            check_stream_interval: Duration::from_millis(500),
            check_stream: true,
            stop_stream: false,
        }
    }

    #[inline]
    pub fn handle_errors(&mut self, err_fn: impl FnMut(cpal::StreamError)) {
        self.error_queue
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain(..)
            .for_each(err_fn)
    }

    /// Starts the audio thread.
    pub fn start_audio_thread(
        &mut self,
        device: Device,
        stream_config: Option<cpal::StreamConfig>,
        sample_format: Option<cpal::SampleFormat>,
        renderer: RendererHandle,
    ) -> Result<(), KaError> {
        let host = cpal::default_host();

        // get output device
        let device = match device {
            Device::Default => host
                .default_output_device()
                .ok_or(KaError::NoOutputDevice)?,
            Device::Name(name) => host
                .output_devices()?
                .find(|d| d.name().unwrap_or_default() == name)
                .ok_or(KaError::NoOutputDevice)?,
            Device::Custom(device) => device,
        };

        // get stream config
        let (config, sample_format) = if let Some(config) = stream_config {
            let sample_format = if let Some(sample_format) = sample_format {
                sample_format
            } else {
                let config = device.default_output_config()?;
                config.sample_format()
            };

            (config, sample_format)
        } else {
            let config = device.default_output_config()?;
            let sample_format = config.sample_format();
            (config.into(), sample_format)
        };

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
            I8 => self.start_stream::<i8>(&device, &config, renderer, custom_device)?,
            I16 => self.start_stream::<i16>(&device, &config, renderer, custom_device)?,
            // I24 => self.start_stream::<I24>(&device, &conf, I24.into(), renderer,custom_device)?,
            I32 => self.start_stream::<i32>(&device, &config, renderer, custom_device)?,
            // I48 => self.start_stream::<I48>(&device, &conf, I48.into(), renderer,custom_device)?,
            I64 => self.start_stream::<i64>(&device, &config, renderer, custom_device)?,
            U8 => self.start_stream::<u8>(&device, &config, renderer, custom_device)?,
            U16 => self.start_stream::<u16>(&device, &config, renderer, custom_device)?,
            // U24 => self.start_stream::<U24>(&device, &conf, U24.into(), renderer,custom_device)?,
            U32 => self.start_stream::<u32>(&device, &config, renderer, custom_device)?,
            // U48 => self.start_stream::<U48>(&device, &conf, U48.into(), renderer,custom_device)?,
            U64 => self.start_stream::<u64>(&device, &config, renderer, custom_device)?,
            F32 => self.start_stream::<f32>(&device, &config, renderer, custom_device)?,
            F64 => self.start_stream::<f64>(&device, &config, renderer, custom_device)?,
            sample_format => return Err(KaError::UnsupportedSampleFormat(sample_format)),
        }

        Ok(())
    }

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
            if let cpal::StreamError::DeviceNotAvailable = err {
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

    fn start_stream<T>(
        &mut self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        renderer: RendererHandle,
        custom_device: bool,
    ) -> Result<(), KaError>
    where
        T: SizedSample + FromSample<f32>,
    {
        // update the renderer's sample rate
        renderer.guard().sample_rate = config.sample_rate.0;

        let channels = config.channels as usize; // number of channels
        let error_queue = self.error_queue.clone(); // stream error queue

        // create a clone of the renderer handle so we can move it inside the
        // stream closure
        let renderer_moved = renderer.clone();

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_exact_mut(channels) {
                    // mix next frame
                    let out = renderer_moved.guard().next_frame();

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
                return self.start_audio_thread(Device::Default, None, None, renderer);
            }

            // see if we should stop the stream
            if self.stop_stream {
                self.stop_stream = false;
                drop(stream);
                break;
            }
        }
        Ok(())
    }
}
