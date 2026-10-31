//! Turns the image around a point and pulls it inward, like water draining.

use std::f64::consts::PI;
use std::ops::RangeInclusive;

use super::Warp;
use crate::geometry::{Point, Size, Vec2};
use crate::rng::Rng;

const WAVY_CHANCE: f64 = 0.5;
/// Sine waves from the center to the edge, when wavy.
const WAVES: RangeInclusive<i32> = 1..=15;
/// Radians per frame at the edge.
const WAVY_ROTATION: RangeInclusive<f64> = -0.22..=0.22;
const SMOOTH_ROTATION: RangeInclusive<f64> = -0.1..=0.1;
/// Pull at the edge, as a fraction of the frame height.
const PULL_OF_HEIGHT: RangeInclusive<f64> = 0.05..=0.3;
const OFF_CENTER_CHANCE: f64 = 0.5;
/// Where an off-center swirl sits, as a fraction of the width and height.
const OFF_CENTER: RangeInclusive<f64> = 0.05..=0.95;

#[derive(Clone, Debug)]
pub(super) struct Swirl {
    center: Point,
    /// Distance from the center to the farthest corner.
    reach: f64,
    pull: f64,
    rotation: f64,
    waves: Option<u32>,
}

impl Swirl {
    pub fn random(rng: &mut Rng, size: Size) -> Self {
        let waves = rng.chance(WAVY_CHANCE).then(|| rng.int(WAVES) as u32);
        let rotation = rng.uniform(if waves.is_some() { WAVY_ROTATION } else { SMOOTH_ROTATION });
        let pull = (rng.uniform(PULL_OF_HEIGHT) * size.height as f64).trunc();
        let center = if rng.chance(OFF_CENTER_CHANCE) {
            let x = rng.uniform(OFF_CENTER) * size.width as f64;
            let y = rng.uniform(OFF_CENTER) * size.height as f64;
            Vec2::new(x, y).truncate()
        } else {
            size.center_point()
        };
        Self { center, reach: farthest_corner(center, size), pull, rotation, waves }
    }
}

impl Warp for Swirl {
    fn source(&self, p: Point) -> Point {
        let offset = (p - self.center).to_vec2();
        let distance = offset.length();
        let edge_fraction = distance / self.reach;
        let pulled = distance - edge_fraction.powi(3) * self.pull;
        let turn = match self.waves {
            Some(waves) => (f64::from(waves) * edge_fraction * PI).sin(),
            None => edge_fraction,
        };
        let angle = offset.angle() + turn * self.rotation;
        let moved = Vec2::from_angle(angle) * pulled;
        moved.truncate() + self.center
    }
}

fn farthest_corner(center: Point, size: Size) -> f64 {
    let (width, height) = size.dims();
    let dx = center.x.max(width - center.x);
    let dy = center.y.max(height - center.y);
    f64::from(dx).hypot(f64::from(dy)).max(1.0)
}
