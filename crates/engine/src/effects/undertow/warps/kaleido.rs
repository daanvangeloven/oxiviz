//! Splits the frame into mirrored pieces. One piece runs an inner warp; the others copy it, so
//! the pattern grows symmetrically.

use std::ops::RangeInclusive;

use super::{Outside, Scroll, Swirl, Warp, Weave};
use crate::geometry::{Point, Size, Vec2};
use crate::rng::Rng;

/// Radians per frame for the rotating inner warp.
const ROTATION: RangeInclusive<f64> = -0.2..=0.2;
/// Tiles: radius of the circle in the middle that keeps the inner warp.
const CIRCLE_RADIUS: RangeInclusive<i32> = 5..=254;
/// Tiles: the circle is shrunk into tiles of this size (plus a 2 pixel gap).
const TILE_SIZE: RangeInclusive<i32> = 3..=52;
const TILE_GAP: i32 = 2;
/// Checker: most checkers use 1 pixel cells, a fine mesh.
const FINE_CHECKER_CHANCE: f64 = 0.9;
const CHECKER_CELL: RangeInclusive<i32> = 1..=300;

/// The warp run inside the piece that the other pieces copy.
#[derive(Clone, Debug)]
enum Inner {
    Swirl(Swirl),
    Scroll(Scroll),
    Weave(Weave),
    Rotate { angle: f64 },
}

#[derive(Clone, Copy, Debug)]
enum Symmetry {
    /// The top-left quarter is warped; the others mirror it.
    Quadrants,
    /// The diagonals cut the frame into four triangles; the left half of the top one is warped
    /// and the rest is folded onto it.
    Triangles,
    /// Outside a circle the image reflects back inward; inside, it's [`Symmetry::Quadrants`].
    Ring { radius: f64 },
    /// A circle in the middle is warped; outside it, the circle repeats as small tiles.
    Tiles { circle_radius: i32, tile_size: i32 },
    /// Every other cell of a checkerboard is warped; the others keep their pixels.
    Checker { cell: i32 },
}

#[derive(Clone, Debug)]
pub(super) struct Kaleido {
    size: (i32, i32),
    center: Point,
    symmetry: Symmetry,
    inner: Inner,
}

impl Kaleido {
    pub fn random(rng: &mut Rng, size: Size) -> Self {
        let inner = match rng.int(0..=3) {
            0 => Inner::Swirl(Swirl::random(rng, size)),
            1 => Inner::Scroll(Scroll::random(rng, size)),
            2 => Inner::Weave(Weave::random(rng, size)),
            _ => Inner::Rotate { angle: rng.uniform(ROTATION) },
        };
        let symmetry = match rng.int(0..=4) {
            0 => Symmetry::Quadrants,
            1 => Symmetry::Triangles,
            2 => Symmetry::Ring { radius: size.width.min(size.height) as f64 / 2.0 },
            3 => Symmetry::Tiles { circle_radius: rng.int(CIRCLE_RADIUS), tile_size: rng.int(TILE_SIZE) },
            _ => {
                let cell = if rng.chance(FINE_CHECKER_CHANCE) { 1 } else { rng.int(CHECKER_CELL) };
                Symmetry::Checker { cell }
            }
        };
        Self { size: size.dims(), center: size.center_point(), symmetry, inner }
    }

    fn inner(&self, p: Point) -> Point {
        match &self.inner {
            Inner::Swirl(warp) => warp.source(p),
            Inner::Scroll(warp) => warp.source(p),
            Inner::Weave(warp) => warp.source(p),
            Inner::Rotate { angle } => rotate(p, self.center, *angle),
        }
    }

    fn quadrants(&self, p: Point) -> Point {
        let (width, height) = self.size;
        let c = self.center;
        if p.x <= c.x && p.y <= c.y {
            return self.inner(p);
        }
        Point::new(if p.x > c.x { width - p.x } else { p.x }, if p.y > c.y { height - p.y } else { p.y })
    }

    fn triangles(&self, p: Point) -> Point {
        let (width, height) = self.size;
        let c = self.center;
        let above_falling_diagonal = width * p.y < height * p.x;
        let above_rising_diagonal = width * p.y < height * (width - p.x);
        match (above_falling_diagonal, above_rising_diagonal) {
            // Top triangle: its left half is the warped piece, the right half mirrors it.
            (true, true) if p.x < c.x => self.inner(p),
            (true, true) => Point::new(2 * c.x - p.x, p.y),
            // Right triangle.
            (true, false) => Point::new(width - p.x + c.x, (p.y - c.y).abs()),
            // Bottom triangle: flipped upside down.
            (false, false) => Point::new(if p.x <= c.x { p.x } else { 2 * c.x - p.x }, height - p.y),
            // Left triangle.
            (false, true) => Point::new(c.x - p.x, (p.y - c.y).abs()),
        }
    }

    fn ring(&self, p: Point, radius: f64) -> Point {
        let offset = (p - self.center).to_vec2();
        let distance = offset.length();
        if distance < radius {
            return self.quadrants(p);
        }
        let reflected = 2.0 * radius - distance;
        (Vec2::from_angle(offset.angle()) * reflected).truncate() + self.center
    }

    fn tiles(&self, p: Point, circle_radius: i32, tile_size: i32) -> Point {
        let distance = (p - self.center).to_vec2().length();
        if distance <= f64::from(circle_radius) {
            return self.inner(p);
        }
        let period = tile_size + TILE_GAP;
        let scale = f64::from(2 * circle_radius) / f64::from(tile_size);
        let circle_corner = self.center - Point::new(circle_radius, circle_radius);
        let in_tile = Vec2::new(f64::from(p.x % period), f64::from(p.y % period));
        (in_tile * scale).round() + circle_corner
    }
}

impl Warp for Kaleido {
    fn source(&self, p: Point) -> Point {
        match self.symmetry {
            Symmetry::Quadrants => self.quadrants(p),
            Symmetry::Triangles => self.triangles(p),
            Symmetry::Ring { radius } => self.ring(p, radius),
            Symmetry::Tiles { circle_radius, tile_size } => self.tiles(p, circle_radius, tile_size),
            Symmetry::Checker { cell } => {
                let odd = |v: i32| (v / cell) % 2 == 1;
                if odd(p.x) || odd(p.y) { p } else { self.inner(p) }
            }
        }
    }

    fn combinable(&self) -> bool {
        false
    }

    fn outside(&self) -> Outside {
        Outside::TopLeft
    }
}

fn rotate(p: Point, center: Point, angle: f64) -> Point {
    let offset = (p - center).to_vec2();
    (Vec2::from_angle(offset.angle() + angle) * offset.length()).truncate() + center
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kaleido(symmetry: Symmetry) -> Kaleido {
        let size = Size::new(100, 60);
        Kaleido {
            size: size.dims(),
            center: size.center_point(),
            symmetry,
            inner: Inner::Rotate { angle: 0.0 },
        }
    }

    #[test]
    fn quadrants_mirror_into_the_top_left() {
        let warp = kaleido(Symmetry::Quadrants);
        assert_eq!(warp.source(Point::new(90, 10)), Point::new(10, 10));
        assert_eq!(warp.source(Point::new(10, 50)), Point::new(10, 10));
        assert_eq!(warp.source(Point::new(90, 50)), Point::new(10, 10));
    }

    #[test]
    fn checker_keeps_odd_cells() {
        let warp = kaleido(Symmetry::Checker { cell: 10 });
        assert_eq!(warp.source(Point::new(15, 3)), Point::new(15, 3));
    }
}
