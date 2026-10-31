use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};
use oxiviz_engine::Size;

/// Where rendered frames go.
pub enum FrameOutput {
    /// Encoded to a video file with the input's audio, by ffmpeg.
    Video { ffmpeg: Child, frames: ChildStdin },
    /// Raw frames on stdout, for a caller that encodes them itself.
    Raw(io::Stdout),
}

pub struct VideoSettings<'a> {
    pub audio: &'a Path,
    pub output: &'a Path,
    pub size: Size,
    pub output_size: Size,
    pub fps: u32,
}

impl FrameOutput {
    pub fn raw() -> Self {
        FrameOutput::Raw(io::stdout())
    }

    pub fn video(settings: &VideoSettings<'_>) -> Result<Self> {
        let mut ffmpeg =
            Command::new("ffmpeg").args(encode_args(settings)).stdin(Stdio::piped()).spawn().map_err(
                |err| match err.kind() {
                    ErrorKind::NotFound => {
                        anyhow!("ffmpeg not found: install it and make sure it is on PATH")
                    }
                    _ => anyhow::Error::new(err).context("couldn't start ffmpeg"),
                },
            )?;
        let frames = ffmpeg.stdin.take().context("ffmpeg has no stdin")?;
        Ok(FrameOutput::Video { ffmpeg, frames })
    }

    pub fn write(&mut self, frame: &[u8]) -> Result<()> {
        match self {
            FrameOutput::Video { frames, .. } => {
                frames.write_all(frame).context("ffmpeg stopped accepting frames")
            }
            FrameOutput::Raw(stdout) => stdout.write_all(frame).context("couldn't write to stdout"),
        }
    }

    /// Flushes the last frames and, for a video, waits for ffmpeg to finish the file.
    pub fn finish(self) -> Result<()> {
        match self {
            FrameOutput::Video { mut ffmpeg, frames } => {
                drop(frames);
                let status = ffmpeg.wait().context("ffmpeg didn't finish")?;
                if !status.success() {
                    bail!("ffmpeg failed ({status})");
                }
                Ok(())
            }
            FrameOutput::Raw(mut stdout) => stdout.flush().context("couldn't write to stdout"),
        }
    }
}

/// Arguments for ffmpeg to encode raw frames from stdin, with the soundtrack from the input.
/// H.264 in yuv420p plays everywhere.
fn encode_args(settings: &VideoSettings<'_>) -> Vec<String> {
    let VideoSettings { audio, output, size, output_size, fps } = settings;
    let words = |text: &str| text.split_whitespace().map(String::from).collect::<Vec<_>>();

    let mut args = words("-nostdin -hide_banner -loglevel error -y");
    args.extend(words(&format!(
        "-f rawvideo -pix_fmt bgr0 -s {}x{} -r {fps} -i pipe:0",
        size.width, size.height
    )));
    args.extend(["-i".into(), audio.display().to_string()]);
    args.extend(words("-map 0:v:0 -map 1:a:0"));
    if output_size != size {
        let scale = format!("scale={}:{}:flags=bicubic", output_size.width, output_size.height);
        args.extend(["-vf".into(), scale]);
    }
    args.extend(words("-c:v libx264 -preset veryfast -crf 20 -pix_fmt yuv420p"));
    args.extend(words("-c:a aac -b:a 192k -shortest -movflags +faststart"));
    args.push(output.display().to_string());
    args
}
