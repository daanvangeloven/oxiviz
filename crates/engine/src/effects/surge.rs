//! Zooms in a little and bobs gently. On a big beat it punches far in, then eases back out.

use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use crate::effects::{Effect, EffectInput, Fade};
use crate::frame::Frame;
use crate::geometry::Size;
use crate::rng::Rng;

/// Fraction of the frame shown right after a big beat.
const BEAT_VIEW: RangeInclusive<f64> = 0.13..=0.25;
/// How far the view bobs in.
const BOB_DEPTH: RangeInclusive<f64> = 0.05..=0.2;
const BOB_FRAMES: RangeInclusive<i32> = 25..=55;
/// Each frame the view moves 1/16th of the way to its target.
const SMOOTHING_WEIGHT: f64 = 15.0;
const SMOOTHING_TOTAL: f64 = 16.0;

pub(crate) struct Surge {
    beat_view: f64,
    bob_depth: f64,
    bob_frames: i32,
    frame: i32,
    view: f64,
}

impl Surge {
    pub fn new() -> Self {
        Self { beat_view: 1.0, bob_depth: 0.0, bob_frames: 1, frame: 0, view: 1.0 }
    }

    fn bob_target(&self) -> f64 {
        let phase = f64::from(self.frame % self.bob_frames) / f64::from(self.bob_frames);
        1.0 - self.bob_depth + self.bob_depth * (TAU * phase).sin()
    }
}

impl Effect for Surge {
    fn randomize(&mut self, rng: &mut Rng, _size: Size) {
        self.beat_view = rng.uniform(BEAT_VIEW);
        self.bob_depth = rng.uniform(BOB_DEPTH);
        self.bob_frames = rng.int(BOB_FRAMES);
        self.frame = 0;
        self.view = 1.0;
    }

    fn render(&mut self, frame: &mut Frame, input: &EffectInput<'_>, _rng: &mut Rng) {
        let fading_out = match input.fade {
            Fade::Out { progress } => Some(progress),
            _ => None,
        };
        if input.levels.big_beat && fading_out.is_none() {
            self.view = self.beat_view;
            self.frame = 0;
        }

        // While fading out, the target drifts to the full view and the smoothing loosens, so the
        // view is back to normal by the time the effect ends.
        let (target, weight) = match fading_out {
            Some(progress) => {
                let target = self.bob_target() * (1.0 - progress) + progress;
                (target, SMOOTHING_WEIGHT * (1.0 - progress))
            }
            None => (self.bob_target(), SMOOTHING_WEIGHT),
        };
        self.view = ((target * (SMOOTHING_TOTAL - weight) + self.view * weight) / SMOOTHING_TOTAL).min(1.0);
        self.frame += 1;

        frame.set_view_scale(self.view);
    }
}
