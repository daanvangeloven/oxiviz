use std::ops::{Add, Mul, Sub};

/// Size of a frame in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn area(self) -> usize {
        self.width * self.height
    }

    /// Width and height as signed pixel coordinates.
    pub(crate) fn dims(self) -> (i32, i32) {
        (self.width as i32, self.height as i32)
    }

    pub(crate) fn center(self) -> Vec2 {
        Vec2::new(self.width as f64 / 2.0, self.height as f64 / 2.0)
    }

    pub(crate) fn center_point(self) -> Point {
        let (width, height) = self.dims();
        Point::new(width / 2, height / 2)
    }

    pub(crate) fn diagonal(self) -> f64 {
        (self.width as f64).hypot(self.height as f64)
    }

    pub(crate) fn contains(self, p: Point) -> bool {
        let (width, height) = self.dims();
        (0..width).contains(&p.x) && (0..height).contains(&p.y)
    }

    /// Index of `p` in a row-major pixel buffer. `p` must be inside.
    pub(crate) fn index(self, p: Point) -> usize {
        debug_assert!(self.contains(p));
        p.y as usize * self.width + p.x as usize
    }
}

/// A pixel position. May lie outside the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const ORIGIN: Point = Point::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(f64::from(self.x), f64::from(self.y))
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

/// A position or direction with sub-pixel precision.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Unit vector pointing at `angle` radians.
    pub fn from_angle(angle: f64) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }

    /// This vector turned 90° (clockwise on screen, where y points down).
    pub fn perpendicular(self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length == 0.0 { self } else { self * (1.0 / length) }
    }

    pub fn lerp(self, to: Vec2, t: f64) -> Self {
        self + (to - self) * t
    }

    /// Nearest pixel, rounding halves away from zero.
    pub fn round(self) -> Point {
        Point::new(self.x.round() as i32, self.y.round() as i32)
    }

    /// Pixel found by dropping the fractions (rounding toward zero).
    pub fn truncate(self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, factor: f64) -> Vec2 {
        Vec2::new(self.x * factor, self.y * factor)
    }
}
