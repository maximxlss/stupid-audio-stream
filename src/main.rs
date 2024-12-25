use std::collections::VecDeque;

use stupid_audio_stream::{
    get_sink_from_string, get_source_from_string, MAX_DATAGRAM
};
use wasapi::initialize_mta;

use anyhow::{Result, anyhow};
use log::warn;
use simplelog::{self, SimpleLogger};

use clap::Parser;

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

    /// Max internal buffer length
    #[arg(short, long, default_value_t = 10000)]
    buffer_limit: usize,

    /// UDP datagram size limit
    #[arg(short, long, default_value_t = 5000)]
    datagram_size: usize,
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

    MAX_DATAGRAM.store(args.datagram_size, std::sync::atomic::Ordering::Relaxed);

    let mut event_handlers = Vec::new();

    let (mut source, possible_event_handler) = get_source_from_string(&args.source)?;
    if let Some(event_handler) = possible_event_handler {
        event_handlers.push(event_handler);
    }

    let (mut sink, possible_event_handler) = get_sink_from_string(&args.sink)?;
    if let Some(event_handler) = possible_event_handler {
        event_handlers.push(event_handler);
    }

    let mut deq = VecDeque::new();

    loop {
        source.read_to_deque(&mut deq)?;
        sink.send_from_deque(&mut deq)?;
        if deq.len() > args.buffer_limit {
            deq.clear();
            warn!("Buffer too full, clearing.");
        }
        for event_handler in &event_handlers {
            event_handler
                .wait_for_event(1000)
                .map_err(|err| anyhow!("Timeout error: {err}"))?;
        }
    }
}
