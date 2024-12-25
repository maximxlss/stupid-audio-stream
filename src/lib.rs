use clap::{command, Parser};
use lazy_static::lazy_static;
use wasapi::WaveFormat;

mod device_management;
pub use device_management::*;

mod sources;
pub use sources::*;

mod sinks;
pub use sinks::*;

lazy_static! {
    static ref DEFAULT_FORMAT: WaveFormat =
        WaveFormat::new(16, 16, &wasapi::SampleType::Int, 48000, 2, None);
}

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
}
