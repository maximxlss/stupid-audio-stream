use std::collections::VecDeque;

use clap::Parser;
use stupid_audio_stream::{Args, HYPOT_AUDIO_ALIGNMENT, sinks, sources};
use wasapi::initialize_mta;

use anyhow::{Result, anyhow};
use log::warn;
use simplelog::{self, SimpleLogger};

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
    let mut source = sources::from_args(&args)?;
    let mut sink = sinks::from_args(&args)?;

    let mut deq = VecDeque::new();

    loop {
        source.recv_to_deque(&mut deq)?;
        sink.send_from_deque(&mut deq)?;
        if deq.len() > args.buffer_limit {
            if args.restart_on_buffer_filled {
                source.restart()?;
                sink.restart()?;
                deq.clear();
                warn!("Buffer too full, restarting source and sink.");
            } else {
                let n_blocks = deq.len() / HYPOT_AUDIO_ALIGNMENT;
                deq.drain(0..(n_blocks * HYPOT_AUDIO_ALIGNMENT));
                warn!("Buffer too full, clearing.");
            }
        }
    }
}
