//! `oxiviz`: renders a music visualizer video for an audio file.

mod args;
mod levels;
mod output;
mod render;

use clap::Parser;

use args::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.levels { levels::run(&args) } else { render::run(&args) }
}
