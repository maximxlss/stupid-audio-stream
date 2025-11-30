use anyhow::Result;
use clap::Parser;

use crate::{sinks::SendAudio, sources::RecvAudio};

pub mod device_utils;
pub mod sinks;
pub mod sources;

/// Program to stream raw audio data between WASAPI devices and UDP sockets
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The source eg. "udp://0.0.0.0:1234" or "mic"
    /// WASAPI devices are found by looking at case-insensitive inclusion of provided name
    pub source: String,

    /// The sink eg. "udp://192.123.123.1:1234" or "speakers"
    /// WASAPI devices are found by looking at case-insensitive inclusion of provided name
    pub sink: String,

    /// Max internal buffer length
    #[arg(short, long, default_value_t = 10000)]
    pub buffer_limit: usize,

    /// UDP datagram size limit
    #[arg(short, long, default_value_t = 5000)]
    pub datagram_size: usize,

    /// Bits per sample
    #[arg(long, default_value_t = 32)]
    pub bits_per_sample: usize,

    /// Sample rate
    #[arg(short, long, default_value_t = 48000)]
    pub sample_rate: usize,

    /// Channels
    #[arg(short, long, default_value_t = 2)]
    pub channels: usize,

    /// Use floating-point samples
    #[arg(long)]
    pub use_float: bool,

    /// Check UDP packet order and loss
    #[arg(long)]
    pub counted_udp: bool,

    /// Restart the sink and source completely if the buffer fills up
    #[arg(long)]
    pub restart_on_buffer_filled: bool,
}

pub trait Restart {
    fn restart(&mut self) -> Result<()>;
}

pub trait SendAudioRestart: SendAudio + Restart {}
impl<T: SendAudio + Restart> SendAudioRestart for T {}

pub trait RecvAudioRestart: RecvAudio + Restart {}
impl<T: RecvAudio + Restart> RecvAudioRestart for T {}

pub const HYPOT_AUDIO_ALIGNMENT: usize = 128; // TODO: use real nBlockAlign
