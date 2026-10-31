//! A spinning polygon or star whose edges are waveforms. It grows with the bass and spins with
//! the treble.

use std::f64::consts::TAU;
use std::ops::RangeInclusive;

use crate::color::{Color, ColorFade, Gradient};
use crate::draw::{Envelope, Scope};
use crate::effects::{Effect, EffectInput};
use crate::frame::Frame;
use crate::geometry::{Size, Vec2};
use crate::rng::Rng;

/// Radius at full bass, as a fraction of half the frame height.
const SCALE: RangeInclusive<f64> = 0.2..=1.0;
const CORNERS: RangeInclusive<i32> = 3..=14;
/// Squares and pentagons are often swapped for something with more corners: one in this many
/// get [`MANY_EXTRA_CORNERS`], the rest [`FEW_EXTRA_CORNERS`].
const MANY_EXTRA_CORNERS_ONE_IN: u32 = 3;
const MANY_EXTRA_CORNERS: i32 = 15;
const FEW_EXTRA_CORNERS: i32 = 2;
/// Radians per frame at full treble.
const MAX_SPIN: RangeInclusive<f64> = -0.3..=0.3;
const COLOR_FADE_FRAMES: RangeInclusive<i32> = 5..=54;
const COLOR_LEGS: u32 = 3;
const EDGE_SCOPE: Scope =
    Scope { max_points: 50, amplitude: 50.0, envelope: Envelope::Flat, mirrored: false, line_width: 1 };

pub(crate) struct Polygram {
    scale: f64,
    corners: u32,
    max_spin: f64,
    angle: f64,
    color_a: ColorFade,
    color_b: ColorFade,
}

impl Polygram {
    pub fn new() -> Self {
        Self {
            scale: 0.0,
            corners: 3,
            max_spin: 0.0,
            angle: 0.0,
            color_a: ColorFade::new(Color::BLACK, Color::BLACK, 1),
            color_b: ColorFade::new(Color::BLACK, Color::BLACK, 1),
        }
    }

    fn corner(&self, center: Vec2, radius: f64, index: u32) -> Vec2 {
        let angle = self.angle + f64::from(index) * TAU / f64::from(self.corners);
        center + Vec2::from_angle(angle) * radius
    }
}

impl Effect for Polygram {
    fn randomize(&mut self, rng: &mut Rng, _size: Size) {
        self.scale = rng.uniform(SCALE);
        let mut corners = rng.int(CORNERS);
        if corners == 4 || corners == 5 {
            corners +=
                if rng.one_in(MANY_EXTRA_CORNERS_ONE_IN) { MANY_EXTRA_CORNERS } else { FEW_EXTRA_CORNERS };
        }
        self.corners = corners as u32;
        self.max_spin = rng.uniform(MAX_SPIN);
        self.color_a = ColorFade::random(rng, COLOR_FADE_FRAMES);
        self.color_b = ColorFade::random(rng, COLOR_FADE_FRAMES);
    }

    fn render(&mut self, frame: &mut Frame, input: &EffectInput<'_>, rng: &mut Rng) {
        let canvas = frame.current_mut();
        let size = canvas.size();
        let radius = input.levels.bass * (size.height as f64 / 2.0) * self.scale;
        self.angle = (self.angle + input.levels.treble * self.max_spin).rem_euclid(TAU);
        self.color_a.step(rng);
        self.color_b.step(rng);

        let colors = Gradient { a: self.color_a.current(), b: self.color_b.current(), legs: COLOR_LEGS };
        let center = size.center();
        for index in 0..self.corners {
            let from = self.corner(center, radius, index);
            let to = self.corner(center, radius, (index + 1) % self.corners);
            EDGE_SCOPE.draw(canvas, input.audio, from, to, colors);
        }
    }
}
