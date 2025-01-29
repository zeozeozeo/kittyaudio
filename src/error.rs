use thiserror::Error;

/// KittyAudio's error type.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum KaError {
    #[error("failed to get output device")]
    NoOutputDevice,
    #[error("failed to get output devices: {0}")]
    #[cfg(feature = "cpal")]
    DeviceError(#[from] cpal::DevicesError),
    #[error("failed to retrieve default stream config: {0}")]
    #[cfg(feature = "cpal")]
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),
    #[error("unsupported sample format {0}")]
    #[cfg(feature = "cpal")]
    UnsupportedSampleFormat(cpal::SampleFormat),
    #[error("failed to build stream: {0}")]
    #[cfg(feature = "cpal")]
    BuildStreamError(#[from] cpal::BuildStreamError),
    #[error("failed to play stream: {0}")]
    #[cfg(feature = "cpal")]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error("an error occured on stream: {0}")]
    #[cfg(feature = "cpal")]
    StreamError(#[from] cpal::StreamError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to get default track as no tracks are present")]
    NoTracksArePresent,
    #[error("failed to get cpal device name: {0}")]
    #[cfg(feature = "cpal")]
    DeviceNameError(#[from] cpal::DeviceNameError),

    // [`Sound`] errors
    #[cfg(feature = "symphonia")]
    #[error("symphonia error: {0}")]
    SymphoniaError(#[from] symphonia::core::errors::Error),
    #[error("unsupported number of channels (got {0}, expected 1 or 2)")]
    UnsupportedNumberOfChannels(u32),
    #[error("failed to get sample rate, or it is invalid")]
    UnknownSampleRate,
}
