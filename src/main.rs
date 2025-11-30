use std::collections::VecDeque;

use clap::Parser;
use stupid_audio_stream::{Args, sinks, sources};
use wasapi::initialize_mta;

use anyhow::{Result, anyhow};
use log::warn;
use simplelog::{self, SimpleLogger};

const HYPOT_AUDIO_ALIGNMENT: usize = 128; // TODO: use real nBlockAlign

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

    if args.buffer_limit < HYPOT_AUDIO_ALIGNMENT * 2 {
        return Err(anyhow!(
            "Buffer limit must be at least {}",
            HYPOT_AUDIO_ALIGNMENT * 2
        ));
    }

    let mut event_handlers = Vec::new();

    let (mut source, possible_event_handler) = sources::from_args(&args)?;
    if let Some(event_handler) = possible_event_handler {
        event_handlers.push(event_handler);
    }

    let (mut sink, possible_event_handler) = sinks::from_args(&args)?;
    if let Some(event_handler) = possible_event_handler {
        event_handlers.push(event_handler);
    }

    let mut deq = VecDeque::new();

    loop {
        source.recv_to_deque(&mut deq)?;
        sink.send_from_deque(&mut deq)?;
        if deq.len() > args.buffer_limit {
            let n_blocks = deq.len() / HYPOT_AUDIO_ALIGNMENT;
            deq.drain(0..(n_blocks * HYPOT_AUDIO_ALIGNMENT));
            warn!("Buffer too full, clearing.");
        }
        for event_handler in &event_handlers {
            // TODO: properly handle multiple handlers
            event_handler
                .wait_for_event(1000)
                .map_err(|err| anyhow!("Timeout error: {err}"))?;
        }
    }
}
