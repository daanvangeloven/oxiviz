//! Slides the image, or stretches it away from an edge, optionally with a sine wobble.

use std::f64::consts::PI;
use std::ops::RangeInclusive;

use super::{Outside, Warp};
use crate::geometry::{Point, Size};
use crate::rng::Rng;

/// Pixels per frame on each axis.
const SHIFT: RangeInclusive<i32> = -3..=3;
/// One in this many scrolls slides; the rest stretch.
const SLIDE_ONE_IN: u32 = 3;
/// Stretch per frame, as a fraction.
const STRETCH: RangeInclusive<f64> = 0.0..=0.1;
const WAVES: RangeInclusive<i32> = 1..=15;
/// The wobble moves pixels up to this many times the shift.
const WOBBLE_FACTOR: f64 = 3.0;

/// Which edge a stretch grows away from, per axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Anchor {
    None,
    Start,
    End,
}

/// The stretch directions to pick from, as (x, y) anchors.
const ANCHORS: [(Anchor, Anchor); 8] = [
    (Anchor::Start, Anchor::None),
    (Anchor::None, Anchor::Start),
    (Anchor::End, Anchor::None),
    (Anchor::None, Anchor::End),
    (Anchor::Start, Anchor::Start),
    (Anchor::End, Anchor::End),
    (Anchor::Start, Anchor::End),
    (Anchor::End, Anchor::Start),
];

#[derive(Clone, Copy, Debug)]
enum Motion {
    Slide,
    Stretch { anchors: (Anchor, Anchor), amount: (f64, f64) },
}

#[derive(Clone, Copy, Debug)]
enum Wobble {
    None,
    /// Rows move sideways along a sine wave down the frame.
    Rows,
    /// Columns move up and down along a sine wave across the frame.
    Columns,
}

#[derive(Clone, Debug)]
pub(super) struct Scroll {
    size: (i32, i32),
    shift: (i32, i32),
    motion: Motion,
    wobble: Wobble,
    waves: i32,
}

impl Scroll {
    pub fn random(rng: &mut Rng, size: Size) -> Self {
        let shift = loop {
            let shift = (rng.int(SHIFT), rng.int(SHIFT));
            if shift != (0, 0) {
                break shift;
            }
        };
        let motion = if rng.one_in(SLIDE_ONE_IN) {
            Motion::Slide
        } else {
            Motion::Stretch {
                anchors: ANCHORS[rng.index(ANCHORS.len())],
                amount: (rng.uniform(STRETCH), rng.uniform(STRETCH)),
            }
        };
        let wobble = match rng.int(0..=2) {
            0 => Wobble::None,
            1 => Wobble::Rows,
            _ => Wobble::Columns,
        };
        Self { size: size.dims(), shift, motion, wobble, waves: rng.int(WAVES) }
    }

    fn wave(&self, position: i32, length: i32) -> f64 {
        (f64::from(position) / f64::from(length) * f64::from(self.waves) * PI).sin()
    }
}

impl Warp for Scroll {
    fn source(&self, p: Point) -> Point {
        let (width, height) = self.size;
        let mut s = match self.motion {
            Motion::Slide => Point::new(p.x + self.shift.0, p.y + self.shift.1),
            Motion::Stretch { anchors, amount } => Point::new(
                stretch(p.x, width, anchors.0, amount.0),
                stretch(p.y, height, anchors.1, amount.1),
            ),
        };
        match self.wobble {
            Wobble::None => {}
            Wobble::Rows => {
                let swing = self.wave(s.y, height) * f64::from(self.shift.1) * WOBBLE_FACTOR;
                s.x += swing.round() as i32;
            }
            Wobble::Columns => {
                let swing = self.wave(s.x, width) * f64::from(self.shift.0) * WOBBLE_FACTOR;
                s.y += swing.round() as i32;
            }
        }
        s
    }

    fn outside(&self) -> Outside {
        Outside::TopLeft
    }
}

/// Scales the distance from the anchored edge by `1 + amount`.
fn stretch(position: i32, length: i32, anchor: Anchor, amount: f64) -> i32 {
    match anchor {
        Anchor::None => position,
        Anchor::Start => ((1.0 + amount) * f64::from(position)) as i32,
        Anchor::End => {
            let from_end = f64::from(length - position - 1);
            position - ((1.0 + amount) * from_end - from_end) as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stretch_keeps_its_anchor_edge_fixed() {
        assert_eq!(stretch(0, 100, Anchor::Start, 0.1), 0);
        assert_eq!(stretch(99, 100, Anchor::End, 0.1), 99);
        assert_eq!(stretch(50, 100, Anchor::Start, 0.1), 55);
    }
}
