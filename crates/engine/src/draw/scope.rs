use std::f64::consts::PI;

use crate::audio_frame::{AudioFrame, WAVEFORM_SAMPLES};
use crate::canvas::Canvas;
use crate::color::Gradient;
use crate::draw::draw_line;
use crate::geometry::Vec2;

/// How a scope's amplitude varies along its length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Envelope {
    /// `loops` sine humps: the wave pinches to nothing between them.
    Sine {
        loops: u32,
    },
    /// Grows from nothing at both ends to full in the middle.
    Triangle,
    Flat,
}

impl Envelope {
    /// The amplitude factor at `t` (`0..=1`) along the line.
    fn at(self, t: f64) -> f64 {
        match self {
            Envelope::Sine { loops } => (f64::from(loops) * t * PI).sin().abs(),
            Envelope::Triangle => 1.0 - (2.0 * t - 1.0).abs(),
            Envelope::Flat => 1.0,
        }
    }
}

/// An oscilloscope trace: the waveform drawn along a line segment, sideways from it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Scope {
    /// The most points the trace uses. Short segments use one per pixel of length.
    pub max_points: usize,
    /// Pixels the trace swings away from the segment at full volume.
    pub amplitude: f64,
    pub envelope: Envelope,
    /// Also draw the trace flipped to the other side of the segment.
    pub mirrored: bool,
    pub line_width: u32,
}

impl Scope {
    pub fn draw(&self, canvas: &mut Canvas, audio: &AudioFrame, from: Vec2, to: Vec2, colors: Gradient) {
        let along = to - from;
        let longest_axis = along.x.abs().max(along.y.abs()) as usize;
        let points = self.max_points.min(longest_axis + 1).min(WAVEFORM_SAMPLES);
        let sideways = along.normalized().perpendicular();

        let trace = |i: usize, side: f64| {
            let t = i as f64 / points as f64;
            let swing = audio.mixed_sample(i) * self.amplitude * self.envelope.at(t);
            (from.lerp(to, t) + sideways * (swing * side)).round()
        };
        let sides: &[f64] = if self.mirrored { &[1.0, -1.0] } else { &[1.0] };

        for &side in sides {
            for i in 1..points {
                let color = colors.at(i as f64 / points as f64);
                draw_line(canvas, trace(i - 1, side), trace(i, side), color, self.line_width);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelopes_at_the_ends_and_middle() {
        assert_eq!(Envelope::Triangle.at(0.0), 0.0);
        assert_eq!(Envelope::Triangle.at(0.5), 1.0);
        assert!((Envelope::Sine { loops: 1 }.at(0.5) - 1.0).abs() < 1e-12);
        assert_eq!(Envelope::Flat.at(0.3), 1.0);
    }
}
