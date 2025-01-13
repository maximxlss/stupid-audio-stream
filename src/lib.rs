use clap::{command, Parser};

pub mod sources;
pub mod sinks;
pub mod network;
pub mod device;
pub mod device_utils;

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

    /// Check UDP packet order and loss
    #[arg(long)]
    pub checked_udp: bool,
}
