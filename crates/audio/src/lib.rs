//! Audio input for the visualizer: decodes a file with ffmpeg and cuts it into per-frame
//! spectrum and waveform data ([`AudioFrame`](oxiviz_engine::AudioFrame)).

mod analyzer;
mod decode;

pub use analyzer::FrameAnalyzer;
pub use decode::{Pcm, SAMPLE_RATE, decode};
