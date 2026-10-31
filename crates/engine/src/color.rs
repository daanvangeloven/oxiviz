use std::f64::consts::FRAC_PI_2;
use std::ops::RangeInclusive;

use crate::rng::Rng;

/// A 24-bit color stored as `0x00RRGGBB`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color(pub u32);

impl Color {
    pub const BLACK: Color = Color(0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    pub const fn r(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub const fn g(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub const fn b(self) -> u8 {
        self.0 as u8
    }

    pub(crate) fn random(rng: &mut Rng) -> Self {
        Self(rng.next_u32() & 0x00FF_FFFF)
    }

    pub(crate) fn inverted(self) -> Self {
        Self(!self.0 & 0x00FF_FFFF)
    }

    /// Moves each channel `t` of the way toward `to`.
    ///
    /// `t` isn't clamped and channels wrap around: loud bass can push TwinOrbs'
    /// shading past 1, which flashes odd colors. That glitch is part of the look.
    pub(crate) fn lerp(self, to: Color, t: f64) -> Self {
        let channel = |from: u8, to: u8| {
            let step = ((f64::from(to) - f64::from(from)) * t).round() as i32;
            from.wrapping_add(step as u8)
        };
        Self::rgb(channel(self.r(), to.r()), channel(self.g(), to.g()), channel(self.b(), to.b()))
    }
}

/// Starts fast and slows down on arrival: maps `0..=1` onto `0..=1` along a quarter sine.
pub(crate) fn ease_out(t: f64) -> f64 {
    (t * FRAC_PI_2).sin()
}

/// How long a [`ColorFade`] takes after its first leg.
const RENEWED_FADE_FRAMES: RangeInclusive<i32> = 1..=100;

/// A color that eases toward a target over a number of frames. On arrival it picks a new random
/// target and goes on.
#[derive(Clone, Debug)]
pub(crate) struct ColorFade {
    from: Color,
    to: Color,
    current: Color,
    frame: i32,
    frames: i32,
}

impl ColorFade {
    pub fn new(from: Color, to: Color, frames: i32) -> Self {
        Self { from, to, current: from, frame: 0, frames }
    }

    /// A fade between two random colors, taking a random number of frames from `frames`.
    pub fn random(rng: &mut Rng, frames: RangeInclusive<i32>) -> Self {
        let frames = rng.int(frames);
        Self::new(Color::random(rng), Color::random(rng), frames)
    }

    pub fn current(&self) -> Color {
        self.current
    }

    /// Advances one frame.
    pub fn step(&mut self, rng: &mut Rng) {
        self.frame += 1;
        if self.frame > self.frames {
            *self = Self::new(self.current, Color::random(rng), rng.int(RENEWED_FADE_FRAMES));
        } else {
            let t = f64::from(self.frame) / f64::from(self.frames);
            self.current = self.from.lerp(self.to, ease_out(t));
        }
    }
}

/// Colors along a line: eases from `a` to `b`, then back to `a`, and so on, for `legs` legs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Gradient {
    pub a: Color,
    pub b: Color,
    pub legs: u32,
}

impl Gradient {
    /// Color at `t` (`0..=1`) along the line.
    pub fn at(self, t: f64) -> Color {
        let position = t.clamp(0.0, 1.0) * f64::from(self.legs.max(1));
        let leg = position.floor();
        let (from, to) = if leg as u32 % 2 == 0 { (self.a, self.b) } else { (self.b, self.a) };
        from.lerp(to, ease_out(position - leg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Color = Color::rgb(255, 0, 0);
    const BLUE: Color = Color::rgb(0, 0, 255);

    #[test]
    fn lerp_hits_both_ends() {
        assert_eq!(RED.lerp(BLUE, 0.0), RED);
        assert_eq!(RED.lerp(BLUE, 1.0), BLUE);
        assert_eq!(RED.lerp(BLUE, 0.5), Color::rgb(127, 0, 128));
    }

    #[test]
    fn gradient_goes_there_and_back() {
        let gradient = Gradient { a: RED, b: BLUE, legs: 2 };
        assert_eq!(gradient.at(0.0), RED);
        assert_eq!(gradient.at(0.5), BLUE);
        assert_eq!(gradient.at(1.0), RED);
    }

    #[test]
    fn color_fade_arrives_then_renews() {
        let mut rng = Rng::new(3);
        let mut fade = ColorFade::new(RED, BLUE, 4);
        for _ in 0..4 {
            fade.step(&mut rng);
        }
        assert_eq!(fade.current(), BLUE);
        fade.step(&mut rng);
        assert_eq!(fade.current(), BLUE, "a renewed fade starts where the last one ended");
    }
}
