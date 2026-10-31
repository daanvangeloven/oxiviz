//! The visualizer engine: turns per-frame audio data into pixels. It does no I/O.
//!
//! A [scheduler](scheduler) keeps a feedback warp and a few overlays running, each for a random
//! number of frames, and replaces them with freshly randomized ones when their time is up.
//! Everything random comes from one seeded [`Rng`](rng::Rng), so a seed always renders the same video.

mod audio_frame;
mod canvas;
mod color;
mod draw;
mod effects;
mod frame;
mod geometry;
mod levels;
mod rng;
mod scheduler;
mod visualizer;

pub use audio_frame::{AudioFrame, Channel, SPECTRUM_BINS, WAVEFORM_SAMPLES};
pub use canvas::Canvas;
pub use color::Color;
pub use geometry::Size;
pub use levels::{LevelTracker, Levels};
pub use visualizer::{Config, Visualizer};
