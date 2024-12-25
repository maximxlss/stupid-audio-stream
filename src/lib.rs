use lazy_static::lazy_static;
use wasapi::WaveFormat;

mod device_management;
pub use device_management::*;

mod sources;
pub use sources::*;

mod sinks;
pub use sinks::*;

const MAX_DATAGRAM: usize = 5000;

lazy_static! {
    static ref DEFAULT_FORMAT: WaveFormat =
        WaveFormat::new(16, 16, &wasapi::SampleType::Int, 48000, 2, None);
}
