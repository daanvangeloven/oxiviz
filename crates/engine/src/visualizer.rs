use crate::audio_frame::AudioFrame;
use crate::canvas::Canvas;
use crate::color::Color;
use crate::frame::Frame;
use crate::geometry::{Point, Size};
use crate::levels::LevelTracker;
use crate::rng::Rng;
use crate::scheduler::Scheduler;

/// Smallest frame the effects can work with.
const MIN_SIZE: Size = Size::new(16, 16);

#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Size the effects render at. Effect sizes are in pixels and were made for small frames
    /// (around 640×360), so render small and scale the video up afterwards.
    pub size: Size,
    pub seed: u64,
    pub background: Color,
}

/// Renders frames one at a time, each from the audio of that frame.
pub struct Visualizer {
    frame: Frame,
    scheduler: Scheduler,
    levels: LevelTracker,
    rng: Rng,
    /// The zoomed-in view, when an effect zooms.
    view: Canvas,
}

impl Visualizer {
    /// # Panics
    /// If `config.size` is smaller than 16×16 pixels.
    pub fn new(config: Config) -> Self {
        assert!(
            config.size.width >= MIN_SIZE.width && config.size.height >= MIN_SIZE.height,
            "frames must be at least {}×{} pixels",
            MIN_SIZE.width,
            MIN_SIZE.height,
        );
        Self {
            frame: Frame::new(config.size, config.background),
            scheduler: Scheduler::new(),
            levels: LevelTracker::default(),
            rng: Rng::new(config.seed),
            view: Canvas::new(config.size, config.background),
        }
    }

    /// Renders the next frame and returns it.
    pub fn render(&mut self, audio: &AudioFrame) -> &Canvas {
        let levels = self.levels.update(audio);
        self.frame.begin();
        self.scheduler.render(&mut self.frame, audio, levels, &mut self.rng);

        let scale = self.frame.view_scale();
        if scale >= 1.0 {
            return self.frame.current();
        }
        zoom(self.frame.current(), &mut self.view, scale);
        &self.view
    }
}

/// Fills `target` with the centered `scale` part of `source`, stretched with nearest-neighbor
/// sampling, so it looks blocky.
fn zoom(source: &Canvas, target: &mut Canvas, scale: f64) {
    let size = source.size();
    let offset_x = size.width as f64 * (1.0 - scale) / 2.0;
    let offset_y = size.height as f64 * (1.0 - scale) / 2.0;
    let width = size.width;
    for (index, pixel) in target.pixels_mut().iter_mut().enumerate() {
        let (x, y) = (index % width, index / width);
        let source_x = (offset_x + x as f64 * scale) as i32;
        let source_y = (offset_y + y as f64 * scale) as i32;
        *pixel = source.get(Point::new(source_x, source_y)).unwrap_or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_frames(seed: u64, frames: usize) -> Vec<Color> {
        let config = Config { size: Size::new(80, 48), seed, background: Color::BLACK };
        let mut visualizer = Visualizer::new(config);
        let mut audio = AudioFrame::silent();
        let mut last = Vec::new();
        for i in 0..frames {
            // A beat every 15 frames.
            let value = if i % 15 == 0 { 230 } else { 60 };
            audio.spectrum.iter_mut().for_each(|channel| channel.fill(value));
            audio.waveform[0]
                .iter_mut()
                .enumerate()
                .for_each(|(j, s)| *s = (128 + (j % 64) as i32 - 32) as u8);
            last = visualizer.render(&audio).pixels().to_vec();
        }
        last
    }

    #[test]
    fn same_seed_renders_the_same_frames() {
        assert_eq!(render_frames(42, 300), render_frames(42, 300));
    }

    #[test]
    fn different_seeds_render_different_frames() {
        assert_ne!(render_frames(1, 300), render_frames(2, 300));
    }

    #[test]
    fn zoom_of_one_copies() {
        let size = Size::new(20, 20);
        let mut source = Canvas::new(size, Color::BLACK);
        source.set(Point::new(3, 4), Color::rgb(1, 2, 3));
        let mut target = Canvas::new(size, Color::BLACK);
        zoom(&source, &mut target, 1.0);
        assert_eq!(target.pixels(), source.pixels());
    }
}
