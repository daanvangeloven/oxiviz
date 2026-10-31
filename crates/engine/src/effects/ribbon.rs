//! A long waveform line through the center of the screen that spins with the music, sometimes
//! with a second line crossing it.

use std::f64::consts::{FRAC_PI_2, PI};
use std::ops::RangeInclusive;

use crate::canvas::Canvas;
use crate::color::{Color, ColorFade, Gradient};
use crate::draw::{Envelope, Scope};
use crate::effects::{Effect, EffectInput};
use crate::frame::Frame;
use crate::geometry::{Size, Vec2};
use crate::rng::Rng;

const POINTS: RangeInclusive<i32> = 10..=521;
/// Line widths to choose from.
const LINE_WIDTHS: [u32; 3] = [1, 2, 3];
/// Draws when picking the envelope, biasing toward the sine envelope.
const ENVELOPE_DRAWS: u32 = 3;
const SINE_LOOPS: RangeInclusive<i32> = 1..=4;
/// Wave amplitude as a fraction of the frame height.
const AMPLITUDE_OF_HEIGHT: RangeInclusive<f64> = 0.1..=0.425;
/// Radians per frame.
const SPIN: RangeInclusive<f64> = -0.13..=0.13;
const BASS_STRETCH_CHANCE: f64 = 0.5;
const MIRRORED_ONE_IN: u32 = 3;
const CROSS_LINE_ONE_IN: u32 = 3;
const CROSS_ANGLE: RangeInclusive<f64> = -FRAC_PI_2..=FRAC_PI_2;
/// A drifting cross line turns at this fraction of the main line's spin.
const CROSS_DRIFT_FACTOR: f64 = 0.5;
const COLOR_FADE_FRAMES: RangeInclusive<i32> = 5..=54;

/// How the spin speed follows the bass.
#[derive(Clone, Copy, Debug)]
enum SpinResponse {
    Squared,
    Linear,
    Constant,
}

impl SpinResponse {
    fn factor(self, bass: f64) -> f64 {
        match self {
            SpinResponse::Squared => bass * bass,
            SpinResponse::Linear => bass,
            SpinResponse::Constant => 1.0,
        }
    }
}

/// The optional second line, at an angle to the main one.
#[derive(Clone, Copy, Debug)]
enum CrossLine {
    /// The angle between the lines changes with the bass.
    Drifting,
    /// The angle between the lines stays the same.
    Fixed,
    /// Always at a right angle to the main line.
    Perpendicular,
}

pub(crate) struct Ribbon {
    scope: Scope,
    stretch_with_bass: bool,
    spin: f64,
    spin_response: SpinResponse,
    angle: f64,
    cross_line: Option<CrossLine>,
    cross_angle: f64,
    color_legs: u32,
    color_a: ColorFade,
    color_b: ColorFade,
}

impl Ribbon {
    pub fn new() -> Self {
        Self {
            scope: Scope {
                max_points: 0,
                amplitude: 0.0,
                envelope: Envelope::Flat,
                mirrored: false,
                line_width: 1,
            },
            stretch_with_bass: false,
            spin: 0.0,
            spin_response: SpinResponse::Constant,
            angle: 0.0,
            cross_line: None,
            cross_angle: 0.0,
            color_legs: 1,
            color_a: ColorFade::new(Color::BLACK, Color::BLACK, 1),
            color_b: ColorFade::new(Color::BLACK, Color::BLACK, 1),
        }
    }

    /// Draws a wave line `length` long through the center of the frame, pointing at `angle`.
    fn draw_line(&self, canvas: &mut Canvas, input: &EffectInput<'_>, angle: f64, length: f64) {
        let center = canvas.size().center();
        let half = Vec2::from_angle(angle) * (length / 2.0);
        let colors = Gradient { a: self.color_a.current(), b: self.color_b.current(), legs: self.color_legs };
        self.scope.draw(canvas, input.audio, center - half, center + half, colors);
    }
}

impl Effect for Ribbon {
    fn randomize(&mut self, rng: &mut Rng, size: Size) {
        let sine_loops = rng.int(SINE_LOOPS) as u32;
        let envelope = match rng.int_biased_low(0..=2, ENVELOPE_DRAWS) {
            0 => Envelope::Sine { loops: sine_loops },
            1 => Envelope::Triangle,
            _ => Envelope::Flat,
        };
        self.scope = Scope {
            max_points: rng.int(POINTS) as usize,
            amplitude: rng.uniform(AMPLITUDE_OF_HEIGHT) * size.height as f64,
            envelope,
            mirrored: rng.one_in(MIRRORED_ONE_IN),
            line_width: LINE_WIDTHS[rng.index(LINE_WIDTHS.len())],
        };
        self.stretch_with_bass = rng.chance(BASS_STRETCH_CHANCE);
        self.spin = rng.uniform(SPIN);
        // Constant is twice as likely as the others.
        self.spin_response = match rng.int(0..=3) {
            0 => SpinResponse::Squared,
            1 => SpinResponse::Linear,
            _ => SpinResponse::Constant,
        };
        self.cross_line = rng.one_in(CROSS_LINE_ONE_IN).then(|| match rng.int(0..=2) {
            0 => CrossLine::Drifting,
            1 => CrossLine::Fixed,
            _ => CrossLine::Perpendicular,
        });
        self.cross_angle = rng.uniform(CROSS_ANGLE);
        self.color_legs = sine_loops * 2;
        self.color_a = ColorFade::random(rng, COLOR_FADE_FRAMES);
        self.color_b = ColorFade::random(rng, COLOR_FADE_FRAMES);
    }

    fn render(&mut self, frame: &mut Frame, input: &EffectInput<'_>, rng: &mut Rng) {
        let bass = input.levels.bass;
        self.color_a.step(rng);
        self.color_b.step(rng);

        let canvas = frame.current_mut();
        let mut length = canvas.size().diagonal();
        if self.stretch_with_bass {
            length *= bass;
        }
        self.draw_line(canvas, input, self.angle + FRAC_PI_2, length);

        if let Some(cross_line) = self.cross_line {
            let cross_angle = match cross_line {
                CrossLine::Drifting => {
                    self.cross_angle -= self.spin * bass * CROSS_DRIFT_FACTOR;
                    self.angle + self.cross_angle
                }
                CrossLine::Fixed => self.angle + self.cross_angle,
                CrossLine::Perpendicular => self.angle + FRAC_PI_2,
            };
            self.draw_line(canvas, input, cross_angle + FRAC_PI_2, length);
        }

        self.angle = (self.angle + self.spin * self.spin_response.factor(bass)).rem_euclid(2.0 * PI);
    }
}
