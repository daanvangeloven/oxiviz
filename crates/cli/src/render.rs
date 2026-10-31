use std::io::{self, IsTerminal};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use oxiviz_audio::FrameAnalyzer;
use oxiviz_engine::{AudioFrame, Color, Config, Visualizer};

use crate::args::Args;
use crate::output::{FrameOutput, VideoSettings};

pub fn run(args: &Args) -> Result<()> {
    let seed = args.seed.unwrap_or_else(random_seed);
    eprintln!("seed {seed}");

    let pcm = oxiviz_audio::decode(&args.input)?;
    let fps = f64::from(args.fps);
    let mut analyzer = FrameAnalyzer::new(pcm, fps);
    let frame_count = match args.duration {
        Some(seconds) => analyzer.frame_count().min((seconds * fps).ceil() as usize),
        None => analyzer.frame_count(),
    };

    let render_size = args.render_size();
    let output_path = args.output.as_deref().context("no output file given")?;
    let mut output = if output_path.as_os_str() == "-" {
        eprintln!("raw frames {}x{}", render_size.width, render_size.height);
        FrameOutput::raw()
    } else {
        FrameOutput::video(&VideoSettings {
            audio: &args.input,
            output: output_path,
            size: render_size,
            output_size: args.size,
            fps: args.fps,
        })?
    };

    let mut visualizer = Visualizer::new(Config { size: render_size, seed, background: Color::BLACK });
    let mut audio = AudioFrame::silent();
    let mut bytes = Vec::new();
    let mut progress = Progress::new(frame_count);
    for index in 0..frame_count {
        analyzer.analyze(index, &mut audio);
        bytes.clear();
        visualizer.render(&audio).write_bgr0(&mut bytes);
        output.write(&bytes)?;
        progress.update(index + 1);
    }
    output.finish()?;
    progress.done();
    Ok(())
}

/// A seed from the clock, for when none is given.
fn random_seed() -> u64 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    // Keep it short enough to type back in.
    since_epoch.as_nanos() as u64 % 1_000_000_000
}

/// Reports how far rendering is on stderr: a live percentage on a terminal, otherwise a line
/// every [`LOGGED_PERCENT_STEP`] percent so logs stay short.
struct Progress {
    total: usize,
    started: Instant,
    interactive: bool,
    last_percent: Option<usize>,
}

const LOGGED_PERCENT_STEP: usize = 10;

impl Progress {
    fn new(total: usize) -> Self {
        Self { total, started: Instant::now(), interactive: io::stderr().is_terminal(), last_percent: None }
    }

    fn update(&mut self, done: usize) {
        let percent = done * 100 / self.total.max(1);
        let step = if self.interactive { 1 } else { LOGGED_PERCENT_STEP };
        if self.last_percent.is_some_and(|last| percent < last + step) {
            return;
        }
        self.last_percent = Some(percent);
        let line = format!("rendering {percent:3}% ({done}/{} frames)", self.total);
        if self.interactive {
            eprint!("\r{line}");
        } else {
            eprintln!("{line}");
        }
    }

    fn done(&self) {
        let seconds = self.started.elapsed().as_secs_f64();
        let fps = self.total as f64 / seconds.max(f64::EPSILON);
        let newline = if self.interactive { "\n" } else { "" };
        eprintln!("{newline}rendered {} frames in {seconds:.1}s ({fps:.0} fps)", self.total);
    }
}
