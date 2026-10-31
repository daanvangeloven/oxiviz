use std::path::PathBuf;

use clap::Parser;
use oxiviz_engine::Size;

/// Renders a music visualizer video for an audio (or video) file.
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Audio or video file to visualize. Its audio is also the video's soundtrack.
    pub input: PathBuf,

    /// Video file to write, or `-` for raw `bgr0` frames at the render size on stdout (no audio).
    #[arg(short, long, required_unless_present = "levels")]
    pub output: Option<PathBuf>,

    /// Size of the video.
    #[arg(long, default_value = "1280x720", value_parser = parse_size)]
    pub size: Size,

    /// Height the effects render at before scaling up to --size. Effects are sized in pixels
    /// for small frames, so this sets how big they look.
    #[arg(long, default_value_t = 360)]
    pub render_height: usize,

    /// Frames per second. Effects move per frame, so this also sets their speed.
    #[arg(long, default_value_t = 30)]
    pub fps: u32,

    /// Seed for the random choices: the same seed and audio render the same video.
    /// Random if not given.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Only render this many seconds.
    #[arg(long)]
    pub duration: Option<f64>,

    /// Instead of rendering, print the bass level and beat statistics, to check the audio
    /// calibration.
    #[arg(long)]
    pub levels: bool,
}

impl Args {
    /// The size the effects render at: --size scaled down to --render-height, keeping the
    /// aspect ratio. Never larger than --size.
    pub fn render_size(&self) -> Size {
        if self.size.height <= self.render_height {
            return self.size;
        }
        let scale = self.render_height as f64 / self.size.height as f64;
        let even = |value: f64| ((value / 2.0).round() as usize * 2).max(2);
        Size::new(even(self.size.width as f64 * scale), even(self.render_height as f64))
    }
}

/// Parses `WIDTHxHEIGHT`, like `1280x720`.
fn parse_size(text: &str) -> Result<Size, String> {
    let invalid = || format!("expected WIDTHxHEIGHT, like 1280x720, got `{text}`");
    let (width, height) = text.split_once('x').ok_or_else(invalid)?;
    let width: usize = width.parse().map_err(|_| invalid())?;
    let height: usize = height.parse().map_err(|_| invalid())?;
    if width % 2 != 0 || height % 2 != 0 {
        return Err(format!("width and height must be even for video encoding, got `{text}`"));
    }
    Ok(Size::new(width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(cli: &str) -> Args {
        Args::try_parse_from(cli.split_whitespace()).unwrap()
    }

    #[test]
    fn parses_sizes() {
        assert_eq!(parse_size("640x360"), Ok(Size::new(640, 360)));
        assert!(parse_size("640").is_err());
        assert!(parse_size("641x360").is_err());
    }

    #[test]
    fn render_size_keeps_the_aspect_ratio() {
        assert_eq!(args("oxiviz a.mp3 -o b.mp4 --size 1920x1080").render_size(), Size::new(640, 360));
        assert_eq!(args("oxiviz a.mp3 -o b.mp4 --size 1080x1920").render_size(), Size::new(202, 360));
        assert_eq!(args("oxiviz a.mp3 -o b.mp4 --size 320x180").render_size(), Size::new(320, 180));
    }

    #[test]
    fn output_is_only_optional_for_levels() {
        assert!(Args::try_parse_from(["oxiviz", "a.mp3"]).is_err());
        assert!(Args::try_parse_from(["oxiviz", "a.mp3", "--levels"]).is_ok());
    }
}
