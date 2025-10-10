use std::collections::VecDeque;

use log::info;

use anyhow::{Result, anyhow};

use crate::{Args, device_utils};

pub mod device;
pub mod network;

pub trait Source {
    fn read_to_deque(&mut self, buf: &mut VecDeque<u8>) -> Result<()>;
}

pub fn from_args(args: &Args) -> Result<(Box<dyn Source>, Option<wasapi::Handle>)> {
    Ok(if let Some(address) = args.source.strip_prefix("udp://") {
        let buffer_size = args.datagram_size;
        if args.counted_udp {
            let pack = network::CheckedUdpSourcePack::new(address, buffer_size)?;
            info!(
                "Listening on {address} to packets of a most {buffer_size} bytes with loss checks"
            );
            (Box::new(pack), None)
        } else {
            let pack = network::UdpSourcePack::new(address, buffer_size)?;
            info!("Listening on {address} to packets of a most {buffer_size} bytes");
            (Box::new(pack), None)
        }
    } else if let Some(address) = args.source.strip_prefix("idc://") {
        let buffer_size = args.datagram_size;
        let pack = network::IdcSourcePack::new(address, buffer_size)?;
        info!("Listening on {address} to packets of a most {buffer_size} bytes without caring");
        (Box::new(pack), None)
    } else {
        let format = wasapi::WaveFormat::new(
            args.bits_per_sample,
            args.bits_per_sample,
            if args.use_float {
                &wasapi::SampleType::Float
            } else {
                &wasapi::SampleType::Int
            },
            args.sample_rate,
            args.channels,
            None,
        );
        let device = device_utils::find_device_by_name(wasapi::Direction::Capture, &args.source)?;
        let client = device_utils::open_device_with_format(&device, &format)?;
        let capture_client = client
            .get_audiocaptureclient()
            .map_err(|err| anyhow!("Can't get the capture client for device: {err}"))?;
        let event_handle = client
            .set_get_eventhandle()
            .map_err(|err| anyhow!("Couldn't get event handle of device: {err}"))?;
        client
            .start_stream()
            .map_err(|err| anyhow!("Couldn't start stream of device: {err}"))?;

        let name = device
            .get_friendlyname()
            .map_err(|err| anyhow!("Couldn't get device name due to error: {err}"))?;
        info!("Capturing from {name}");
        (Box::new(capture_client), Some(event_handle))
    })
}
