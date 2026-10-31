//! Two shaded balls that drift around, mirrored through the center, swelling with the bass.
//! On a beat they flash, invert their colors and fire a waveform between each other.

use std::f64::consts::{FRAC_PI_2, PI};
use std::ops::RangeInclusive;

use crate::canvas::Canvas;
use crate::color::{Color, ColorFade, Gradient};
use crate::draw::{Envelope, Scope};
use crate::effects::{Effect, EffectInput};
use crate::frame::Frame;
use crate::geometry::{Point, Size, Vec2};
use crate::levels::Levels;
use crate::rng::Rng;

const START_VELOCITY: RangeInclusive<f64> = -2.0..=2.0;
/// Multiplies the velocity every frame the ball doesn't bounce.
const FRICTION: f64 = 0.8;

const BASE_RADIUS: RangeInclusive<f64> = 9.0..=30.0;
const SPRING_STIFFNESS: f64 = 1.0;
const SPRING_MASS: f64 = 1.0;
/// How hard the bass kicks the radius outward.
const BASS_KICK: f64 = 0.8;
const SPRING_DAMPING: RangeInclusive<f64> = 0.5..=0.9;

const COLOR_FADE_FRAMES: RangeInclusive<i32> = 5..=54;
/// Half the width of the bright rim, in pixels from the ball's edge. One cosine period wide.
const RIM_HALF_WIDTH: f64 = PI;

/// Frames the waveform between the balls stays after a beat.
const BURST_FRAMES: u32 = 5;
/// During a burst the first ball's radius is scaled by `frames left / this`.
const BURST_SHRINK_DIVISOR: f64 = 2.5;
/// Shading strength on a beat frame, instead of the bass level.
const BEAT_SHADING: f64 = 0.4;
const BURST_COLOR_LEGS: RangeInclusive<i32> = 1..=6;
const BURST_SCOPE: Scope =
    Scope { max_points: 500, amplitude: 50.0, envelope: Envelope::Flat, mirrored: false, line_width: 1 };

pub(crate) struct TwinOrbs {
    position: Vec2,
    velocity: Vec2,
    radius: Spring,
    ball_color: ColorFade,
    rim_color: ColorFade,
    inverted: bool,
    burst_frames_left: u32,
}

impl TwinOrbs {
    pub fn new() -> Self {
        Self {
            position: Vec2::default(),
            velocity: Vec2::default(),
            radius: Spring::new(0.0, 0.0),
            ball_color: ColorFade::new(Color::BLACK, Color::BLACK, 1),
            rim_color: ColorFade::new(Color::BLACK, Color::BLACK, 1),
            inverted: false,
            burst_frames_left: 0,
        }
    }

    /// Moves the ball and bounces it off the edges. The bass shakes it up.
    fn advance(&mut self, size: Size, levels: Levels, rng: &mut Rng) {
        self.position = self.position + self.velocity;
        self.velocity.x = bounce_or_slow(self.position.x, size.width, self.velocity.x);
        self.velocity.y = bounce_or_slow(self.position.y, size.height, self.velocity.y);
        self.velocity.x += rng.sign() * levels.bass;
        self.velocity.y += rng.sign() * levels.bass;
    }

    fn draw_burst(
        &self,
        canvas: &mut Canvas,
        input: &EffectInput<'_>,
        from: Point,
        to: Point,
        rng: &mut Rng,
    ) {
        let colors = Gradient {
            a: self.ball_color.current(),
            b: self.rim_color.current(),
            legs: rng.int(BURST_COLOR_LEGS) as u32,
        };
        BURST_SCOPE.draw(canvas, input.audio, from.to_vec2(), to.to_vec2(), colors);
    }

    fn draw_ball(&self, canvas: &mut Canvas, center: Point, radius: i32, shading: f64) {
        let ball =
            if self.inverted { self.ball_color.current().inverted() } else { self.ball_color.current() };
        let rim = self.rim_color.current();
        let radius_f = f64::from(radius);
        let reach = radius + RIM_HALF_WIDTH.ceil() as i32;

        for dy in -reach..=reach {
            for dx in -reach..=reach {
                // Positive inside the ball, about 0 at its edge, negative outside.
                let depth = (radius_f * radius_f - f64::from(dx * dx + dy * dy)) / radius_f;
                canvas.update(Point::new(center.x + dx, center.y + dy), |pixel| {
                    let mut pixel = pixel;
                    if depth >= 1.0 {
                        let see_through = ((depth / radius_f).powi(2) * FRAC_PI_2).cos() * shading;
                        pixel = ball.lerp(pixel, see_through);
                    }
                    if depth.abs() < RIM_HALF_WIDTH {
                        pixel = pixel.lerp(rim, (depth.cos() + 1.0) / 2.0);
                    }
                    pixel
                });
            }
        }
    }
}

impl Effect for TwinOrbs {
    fn randomize(&mut self, rng: &mut Rng, size: Size) {
        let (width, height) = size.dims();
        self.position = Point::new(rng.int(0..=width - 1), rng.int(0..=height - 1)).to_vec2();
        self.velocity = Vec2::new(rng.uniform(START_VELOCITY), rng.uniform(START_VELOCITY));
        let base_radius = rng.uniform(BASE_RADIUS);
        self.radius = Spring::new(base_radius, rng.uniform(SPRING_DAMPING));
        self.ball_color = ColorFade::random(rng, COLOR_FADE_FRAMES);
        self.rim_color = ColorFade::random(rng, COLOR_FADE_FRAMES);
        self.inverted = false;
        self.burst_frames_left = 0;
    }

    fn render(&mut self, frame: &mut Frame, input: &EffectInput<'_>, rng: &mut Rng) {
        let levels = input.levels;
        let size = frame.size();
        self.advance(size, levels, rng);

        let mut radius = (self.radius.update(levels.bass) * levels.bass) as i32 + 1;
        let mut shading = levels.bass;
        let mut first_radius = radius;
        if levels.beat {
            radius *= 2;
            first_radius = radius;
            shading = BEAT_SHADING;
            self.inverted = !self.inverted;
            self.burst_frames_left = BURST_FRAMES;
        } else if self.burst_frames_left > 0 {
            let shrink = f64::from(self.burst_frames_left) / BURST_SHRINK_DIVISOR;
            first_radius = (shrink * f64::from(radius)).round() as i32;
        }

        let first = self.position.round();
        let (width, height) = size.dims();
        let second = Point::new(width - first.x, height - first.y);
        let canvas = frame.current_mut();
        if self.burst_frames_left > 0 {
            self.draw_burst(canvas, input, first, second, rng);
            self.burst_frames_left -= 1;
        }
        self.draw_ball(canvas, first, first_radius, shading);
        self.draw_ball(canvas, second, radius, shading);

        self.ball_color.step(rng);
        self.rim_color.step(rng);
    }
}

/// Reverses the velocity when the position has left `0..=limit`, otherwise slows it down.
fn bounce_or_slow(position: f64, limit: usize, velocity: f64) -> f64 {
    if position < 0.0 || position > limit as f64 { -velocity } else { velocity * FRICTION }
}

/// A damped spring pulling the radius back to its rest length, kicked outward by the bass.
struct Spring {
    rest: f64,
    current: f64,
    velocity: f64,
    damping: f64,
}

impl Spring {
    fn new(rest: f64, damping: f64) -> Self {
        Self { rest, current: rest, velocity: 0.0, damping }
    }

    /// Advances one frame and returns the new length.
    fn update(&mut self, bass: f64) -> f64 {
        let pull = (self.rest - self.current) * SPRING_STIFFNESS / SPRING_MASS;
        let kick = BASS_KICK * bass / SPRING_MASS;
        self.velocity = (self.velocity + pull + kick) * self.damping;
        self.current += self.velocity;
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_settles_at_rest_without_bass() {
        let mut spring = Spring::new(20.0, 0.7);
        spring.current = 40.0;
        for _ in 0..200 {
            spring.update(0.0);
        }
        assert!((spring.current - 20.0).abs() < 1e-6);
    }
}
