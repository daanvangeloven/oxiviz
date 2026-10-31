//! Decides which effects run. A cycle is a list of steps; each step keeps a few effects of one
//! kind running for a random number of frames, then swaps in freshly randomized ones.

use std::ops::RangeInclusive;

use crate::audio_frame::AudioFrame;
use crate::effects::{Effect, EffectInput, Fade, Polygram, Ribbon, Surge, TwinOrbs, Undertow};
use crate::frame::Frame;
use crate::levels::Levels;
use crate::rng::Rng;

/// The kinds of effect a step can run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// Full-screen feedback warps.
    Warp,
    /// Shapes drawn on top.
    Overlay,
    /// Changes how much of the frame is shown.
    Zoom,
}

/// One line of the cycle: how many effects of a kind to run, and for how long.
struct StepSpec {
    kind: Kind,
    count: RangeInclusive<i32>,
    /// The count is the lowest of this many draws, so higher values favor fewer effects.
    count_draws: u32,
    /// Frames before the step picks again.
    frames: RangeInclusive<i32>,
}

/// The steps, in drawing order. Overlays come in two steps so they can stack.
const STEPS: [StepSpec; 4] = [
    StepSpec { kind: Kind::Warp, count: 1..=1, count_draws: 1, frames: 90..=500 },
    StepSpec { kind: Kind::Overlay, count: 1..=2, count_draws: 10, frames: 50..=350 },
    StepSpec { kind: Kind::Overlay, count: 1..=3, count_draws: 10, frames: 50..=350 },
    StepSpec { kind: Kind::Zoom, count: 0..=1, count_draws: 3, frames: 50..=400 },
];

/// Frames before the whole cycle restarts and every step picks again.
const CYCLE_FRAMES: RangeInclusive<i32> = 1500..=7500;
/// Picks to try per wanted effect before giving up on it.
const PICK_ATTEMPTS: u32 = 100;
/// An effect fades in and out over this many frames, or a third of its lifetime if shorter.
const FADE_FRAMES: RangeInclusive<i32> = 10..=90;

struct PoolEntry {
    kind: Kind,
    /// Chance the effect is accepted when it comes up in a pick.
    probability: f64,
    effect: Box<dyn Effect>,
    running: Option<Running>,
}

/// An effect's place in a step.
struct Running {
    step: usize,
    fade_frames: i32,
    age: i32,
}

pub(crate) struct Scheduler {
    pool: Vec<PoolEntry>,
    /// Frames left for each step's current picks.
    step_frames_left: [i32; STEPS.len()],
    cycle_frames_left: i32,
}

impl Scheduler {
    pub fn new() -> Self {
        let entry = |kind, probability, effect: Box<dyn Effect>| PoolEntry {
            kind,
            probability,
            effect,
            running: None,
        };
        Self {
            pool: vec![
                entry(Kind::Warp, 1.0, Box::new(Undertow::new())),
                entry(Kind::Overlay, 1.0, Box::new(Ribbon::new())),
                entry(Kind::Overlay, 0.5, Box::new(Polygram::new())),
                entry(Kind::Overlay, 0.9, Box::new(TwinOrbs::new())),
                entry(Kind::Zoom, 0.5, Box::new(Surge::new())),
            ],
            step_frames_left: [0; STEPS.len()],
            cycle_frames_left: 0,
        }
    }

    /// Runs every active effect for one frame, picking new ones where time is up.
    pub fn render(&mut self, frame: &mut Frame, audio: &AudioFrame, levels: Levels, rng: &mut Rng) {
        self.cycle_frames_left -= 1;
        if self.cycle_frames_left <= 0 {
            self.cycle_frames_left = rng.int(CYCLE_FRAMES);
            self.step_frames_left = [0; STEPS.len()];
        }

        for step in 0..STEPS.len() {
            self.step_frames_left[step] -= 1;
            if self.step_frames_left[step] <= 0 {
                self.pick(step, frame, rng);
            }
            let frames_left = self.step_frames_left[step];
            for entry in &mut self.pool {
                let Some(running) = entry.running.as_mut().filter(|running| running.step == step) else {
                    continue;
                };
                let input = EffectInput { audio, levels, fade: running.fade(frames_left) };
                entry.effect.render(frame, &input, rng);
                running.age += 1;
            }
        }
    }

    /// Stops the step's effects and starts a random new set.
    fn pick(&mut self, step: usize, frame: &Frame, rng: &mut Rng) {
        let spec = &STEPS[step];
        for entry in &mut self.pool {
            if entry.running.as_ref().is_some_and(|running| running.step == step) {
                entry.running = None;
            }
        }

        let count = rng.int_biased_low(spec.count.clone(), spec.count_draws);
        let frames = rng.int(spec.frames.clone());
        self.step_frames_left[step] = frames;

        for _ in 0..count {
            let Some(index) = self.find_idle(spec.kind, rng) else {
                break;
            };
            let entry = &mut self.pool[index];
            entry.effect.randomize(rng, frame.size());
            entry.running =
                Some(Running { step, fade_frames: (frames / 3).min(rng.int(FADE_FRAMES)), age: 0 });
        }
    }

    /// A random idle effect of `kind` that passes its probability roll, if one comes up.
    fn find_idle(&self, kind: Kind, rng: &mut Rng) -> Option<usize> {
        (0..PICK_ATTEMPTS).find_map(|_| {
            let index = rng.index(self.pool.len());
            let entry = &self.pool[index];
            let candidate = entry.kind == kind && entry.running.is_none();
            (candidate && rng.unit() <= entry.probability).then_some(index)
        })
    }
}

impl Running {
    fn fade(&self, step_frames_left: i32) -> Fade {
        if self.fade_frames <= 0 {
            Fade::Steady
        } else if self.age < self.fade_frames {
            Fade::In { progress: f64::from(self.age) / f64::from(self.fade_frames) }
        } else if step_frames_left <= self.fade_frames {
            Fade::Out { progress: 1.0 - f64::from(step_frames_left) / f64::from(self.fade_frames) }
        } else {
            Fade::Steady
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::geometry::Size;

    #[test]
    fn first_frame_starts_a_warp_and_an_overlay() {
        let mut scheduler = Scheduler::new();
        let mut frame = Frame::new(Size::new(64, 48), Color::BLACK);
        scheduler.render(&mut frame, &AudioFrame::silent(), Levels::default(), &mut Rng::new(1));

        let running = |kind| scheduler.pool.iter().filter(|e| e.kind == kind && e.running.is_some()).count();
        assert_eq!(running(Kind::Warp), 1);
        assert!(running(Kind::Overlay) >= 1);
    }

    #[test]
    fn fade_goes_in_then_steady_then_out() {
        let running = Running { step: 0, fade_frames: 10, age: 0 };
        assert_eq!(running.fade(100), Fade::In { progress: 0.0 });
        assert_eq!(Running { age: 50, ..running }.fade(100), Fade::Steady);
        assert_eq!(Running { age: 95, ..running }.fade(5), Fade::Out { progress: 0.5 });
    }
}
