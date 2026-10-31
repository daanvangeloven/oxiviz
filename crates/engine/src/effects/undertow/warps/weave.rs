//! Moves stripes or checkered blocks in opposite directions, like threads in a weave.

use std::ops::RangeInclusive;

use super::Warp;
use crate::geometry::{Point, Size};
use crate::rng::Rng;

/// Stripe width in pixels.
const STRIPE: RangeInclusive<i32> = 1..=40;
/// Pixels moved per frame.
const SPEED: RangeInclusive<i32> = 1..=4;
const VERTICAL_ONE_IN: u32 = 4;

#[derive(Clone, Debug)]
pub(super) struct Weave {
    width: i32,
    stripe: i32,
    speed: i32,
    vertical: bool,
}

impl Weave {
    pub fn random(rng: &mut Rng, size: Size) -> Self {
        Self {
            width: size.dims().0,
            stripe: rng.int(STRIPE),
            speed: rng.int(SPEED),
            vertical: rng.one_in(VERTICAL_ONE_IN),
        }
    }
}

impl Warp for Weave {
    fn source(&self, p: Point) -> Point {
        let (stripe, speed) = (self.stripe, self.speed);
        let period = 2 * stripe;
        let (column, row) = (p.x % period, p.y % period);

        if self.vertical {
            // Alternate columns move up and down.
            let dy = if column <= stripe { -speed } else { speed };
            return Point::new(p.x, p.y + dy);
        }
        // Horizontal bands alternate direction; each band's leading triangle moves down instead.
        let moves_sideways = if row > stripe {
            self.width - p.x >= stripe || column <= row - stripe
        } else {
            p.x >= stripe || column <= row
        };
        match (moves_sideways, row > stripe) {
            (true, true) => Point::new(p.x + speed, p.y),
            (true, false) => Point::new(p.x - speed, p.y),
            (false, _) => Point::new(p.x, p.y + speed),
        }
    }
}
