use anyhow::{Result, bail};
use oxiviz_audio::FrameAnalyzer;
use oxiviz_engine::{AudioFrame, LevelTracker};

use crate::args::Args;

/// The bass range the effects and beat thresholds were made for.
const EXPECTED_BASS: &str = "0.3–0.8";

pub fn run(args: &Args) -> Result<()> {
    let fps = f64::from(args.fps);
    let mut analyzer = FrameAnalyzer::new(oxiviz_audio::decode(&args.input)?, fps);
    let mut tracker = LevelTracker::default();
    let mut audio = AudioFrame::silent();

    let mut bass = Vec::with_capacity(analyzer.frame_count());
    let (mut beats, mut big_beats) = (0, 0);
    for index in 0..analyzer.frame_count() {
        analyzer.analyze(index, &mut audio);
        let levels = tracker.update(&audio);
        bass.push(levels.bass);
        beats += usize::from(levels.beat);
        big_beats += usize::from(levels.big_beat);
    }
    if bass.is_empty() {
        bail!("{} has no audio", args.input.display());
    }

    bass.sort_by(f64::total_cmp);
    let percentile = |p: usize| bass[(bass.len() - 1) * p / 100];
    let minutes = bass.len() as f64 / fps / 60.0;
    println!(
        "bass p10 {:.2}  p50 {:.2}  p90 {:.2}  max {:.2}   (aim for about {EXPECTED_BASS})",
        percentile(10),
        percentile(50),
        percentile(90),
        percentile(100)
    );
    println!("beats {:.0}/min  big beats {:.0}/min", beats as f64 / minutes, big_beats as f64 / minutes);
    Ok(())
}
