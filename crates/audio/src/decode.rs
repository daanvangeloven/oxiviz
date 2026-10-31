use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use oxiviz_engine::Channel;

/// Sample rate audio is decoded at. With 1024 spectrum bins, each bin is about 21.5 Hz wide.
pub const SAMPLE_RATE: u32 = 44_100;

const BYTES_PER_SAMPLE: usize = size_of::<f32>();

/// Decoded stereo audio.
#[derive(Clone, Debug, Default)]
pub struct Pcm {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl Pcm {
    pub fn duration_secs(&self) -> f64 {
        self.left.len() as f64 / f64::from(SAMPLE_RATE)
    }

    pub fn channel(&self, channel: Channel) -> &[f32] {
        match channel {
            Channel::Left => &self.left,
            Channel::Right => &self.right,
        }
    }
}

/// Decodes the first audio stream of any file ffmpeg can read. Mono becomes two equal channels.
pub fn decode(path: &Path) -> Result<Pcm> {
    let output = Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-map", "0:a:0", "-ac", "2", "-ar", &SAMPLE_RATE.to_string(), "-f", "f32le", "-"])
        .output()
        .map_err(|err| match err.kind() {
            ErrorKind::NotFound => {
                anyhow::anyhow!("ffmpeg not found: install it and make sure it is on PATH")
            }
            _ => anyhow::Error::new(err).context("couldn't run ffmpeg"),
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffmpeg couldn't decode {}: {}", path.display(), stderr.trim());
    }
    deinterleave(&output.stdout).with_context(|| format!("decoding {}", path.display()))
}

/// Splits little-endian `f32` samples, alternating left and right, into two channels.
fn deinterleave(bytes: &[u8]) -> Result<Pcm> {
    let frame_bytes = 2 * BYTES_PER_SAMPLE;
    if bytes.len() % frame_bytes != 0 {
        bail!("ffmpeg returned a partial sample");
    }
    let sample = |chunk: &[u8]| f32::from_le_bytes(chunk.try_into().expect("chunk is one sample"));
    let (left, right) = bytes
        .chunks_exact(frame_bytes)
        .map(|frame| (sample(&frame[..BYTES_PER_SAMPLE]), sample(&frame[BYTES_PER_SAMPLE..])))
        .unzip();
    Ok(Pcm { left, right })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deinterleaves_left_and_right() {
        let bytes: Vec<u8> = [0.5f32, -0.5, 1.0, 0.0].iter().flat_map(|s| s.to_le_bytes()).collect();
        let pcm = deinterleave(&bytes).unwrap();
        assert_eq!(pcm.left, [0.5, 1.0]);
        assert_eq!(pcm.right, [-0.5, 0.0]);
    }

    #[test]
    fn rejects_partial_samples() {
        assert!(deinterleave(&[0; 6]).is_err());
    }
}
