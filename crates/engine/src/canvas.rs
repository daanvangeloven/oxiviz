use crate::color::Color;
use crate::geometry::{Point, Size};

/// A frame's pixels, row by row.
#[derive(Clone, Debug)]
pub struct Canvas {
    size: Size,
    pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(size: Size, fill: Color) -> Self {
        Self { size, pixels: vec![fill; size.area()] }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    pub(crate) fn pixels_mut(&mut self) -> &mut [Color] {
        &mut self.pixels
    }

    /// The pixel at `p`, or `None` outside the canvas.
    pub(crate) fn get(&self, p: Point) -> Option<Color> {
        self.size.contains(p).then(|| self.pixels[self.size.index(p)])
    }

    /// Sets the pixel at `p`. Does nothing outside the canvas.
    pub(crate) fn set(&mut self, p: Point, color: Color) {
        if self.size.contains(p) {
            let index = self.size.index(p);
            self.pixels[index] = color;
        }
    }

    /// Replaces the pixel at `p` with `f(pixel)`. Does nothing outside the canvas.
    pub(crate) fn update(&mut self, p: Point, f: impl FnOnce(Color) -> Color) {
        if let Some(pixel) = self.get(p) {
            self.set(p, f(pixel));
        }
    }

    pub(crate) fn fill_row(&mut self, y: usize, color: Color) {
        let width = self.size.width;
        self.pixels[y * width..(y + 1) * width].fill(color);
    }

    /// Appends the pixels to `out` as 4 bytes each: blue, green, red, 0 (ffmpeg's `bgr0`).
    pub fn write_bgr0(&self, out: &mut Vec<u8>) {
        out.reserve(self.pixels.len() * 4);
        out.extend(self.pixels.iter().flat_map(|pixel| pixel.0.to_le_bytes()));
    }
}
