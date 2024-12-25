use std::{collections::VecDeque, net::UdpSocket};

use stupid_audio_stream::{find_device_by_name, open_device_with_format, DeviceSinkPack, Sink, Source};
use wasapi::{
    Direction, WaveFormat, initialize_mta,
};

use anyhow::{Result, anyhow};
use log::info;
use simplelog::{self, SimpleLogger};

use clap::Parser;
use lazy_static::lazy_static;


lazy_static! {
    static ref DEFAULT_FORMAT: WaveFormat =
        WaveFormat::new(16, 16, &wasapi::SampleType::Int, 48000, 2, None);
}

/// Program to stream raw audio data between WASAPI devices and UDP sockets
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The source eg. "udp://0.0.0.0:1234" or "mic"
    /// WASAPI devices are found by looking at case-insensitive inclusion of provided name
    source: String,

    /// The sink eg. "udp://192.123.123.1:1234" or "speakers"
    /// WASAPI devices are found by looking at case-insensitive inclusion of provided name
    sink: String,
}

fn main() -> Result<()> {
    SimpleLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_time_offset_to_local()
            .unwrap()
            .build(),
    )?;

    initialize_mta().unwrap();

    let args = Args::parse();

    let mut event_handlers = Vec::new();

    let (mut source, possible_event_handler): (Box<dyn Source>, _) =
        if let Some(address) = args.source.strip_prefix("udp://") {
            let socket = UdpSocket::bind(&address)?;
            info!("Listening on {address}");
            (Box::new(socket), None)
        } else {
            let device = find_device_by_name(Direction::Capture, &args.source)?;
            let client = open_device_with_format(&device, &DEFAULT_FORMAT)?;
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
        };
    if let Some(event_handler) = possible_event_handler {
        event_handlers.push(event_handler);
    }

    let (mut sink, possible_event_handler): (Box<dyn Sink>, _) =
        if let Some(address) = args.sink.strip_prefix("udp://") {
            let socket = UdpSocket::bind("0.0.0.0:13371")?;
            socket.connect(address)?;
            info!("Sending to {address}");
            (Box::new(socket), None)
        } else {
            let device = find_device_by_name(Direction::Render, &args.sink)?;
            let client = open_device_with_format(&device, &DEFAULT_FORMAT)?;
            let render_client = client
                .get_audiorenderclient()
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
            info!("Sending to {name}");
            (
                Box::new(DeviceSinkPack {
                    render_client,
                    client,
                    format: DEFAULT_FORMAT.clone(),
                }),
                Some(event_handle),
            )
        };
    if let Some(event_handler) = possible_event_handler {
        event_handlers.push(event_handler);
    }

    let mut deq = VecDeque::new();

    loop {
        source.read_to_deque(&mut deq)?;
        sink.send_from_deque(&mut deq)?;
        for event_handler in &event_handlers {
            event_handler
                .wait_for_event(1000)
                .map_err(|err| anyhow!("Timeout error: {err}"))?;
        }
    }
}
