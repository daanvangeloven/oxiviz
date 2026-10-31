use rayon::prelude::*;

use super::warps::{Outside, Warp};
use crate::canvas::Canvas;
use crate::color::Color;
use crate::geometry::{Point, Size};

/// For every pixel, the pixel of the previous frame it copies from. Always inside the frame.
#[derive(Clone, Debug)]
pub(super) struct WarpField {
    size: Size,
    sources: Vec<Point>,
}

impl WarpField {
    /// A field where every pixel copies itself.
    pub fn identity(size: Size) -> Self {
        let (width, height) = size.dims();
        let sources = (0..height).flat_map(|y| (0..width).map(move |x| Point::new(x, y))).collect();
        Self { size, sources }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    /// Computes row `y` from `primary`, averaged with `secondary` applied on top if there is one.
    pub fn build_row(&mut self, y: usize, primary: &dyn Warp, secondary: Option<&dyn Warp>) {
        let width = self.size.width;
        for (x, source) in self.sources[y * width..(y + 1) * width].iter_mut().enumerate() {
            let p = Point::new(x as i32, y as i32);
            let mut s = primary.source(p);
            if let Some(secondary) = secondary {
                let combined = secondary.source(s);
                s = Point::new((combined.x + p.x) / 2, (combined.y + p.y) / 2);
            }
            *source = if self.size.contains(s) {
                s
            } else {
                match primary.outside() {
                    Outside::StayPut => p,
                    Outside::TopLeft => Point::ORIGIN,
                }
            };
        }
    }
}

/// Fills `target` with pixels of `source`, taking each from where the field `progress` of the
/// way from `from` to `to` says.
pub(super) fn remap(source: &Canvas, target: &mut Canvas, from: &WarpField, to: &WarpField, progress: f64) {
    let size = source.size();
    let pixels = source.pixels();
    target.pixels_mut().par_chunks_mut(size.width).enumerate().for_each(|(y, row)| {
        let start = y * size.width;
        let from = &from.sources[start..start + size.width];
        let to = &to.sources[start..start + size.width];
        for ((pixel, &a), &b) in row.iter_mut().zip(from).zip(to) {
            let s = a.to_vec2().lerp(b.to_vec2(), progress).round();
            *pixel = pixels[size.index(s)];
        }
    });
}

/// A soft blur: each channel becomes `(3 × the average of its 4 neighbors + itself) / 4`.
/// Pixels on the edge use themselves for the missing neighbors.
pub(super) fn soften(source: &Canvas, target: &mut Canvas) {
    let size = source.size();
    let pixels = source.pixels();
    let (width, height) = (size.width, size.height);
    target.pixels_mut().par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let line = |y: usize| &pixels[y * width..(y + 1) * width];
        let (above, here, below) = (line(y.saturating_sub(1)), line(y), line((y + 1).min(height - 1)));
        for (x, pixel) in row.iter_mut().enumerate() {
            let left = here[x.saturating_sub(1)];
            let right = here[(x + 1).min(width - 1)];
            *pixel = soften_pixel(here[x], [above[x], below[x], left, right]);
        }
    });
}

fn soften_pixel(center: Color, neighbors: [Color; 4]) -> Color {
    let channel = |get: fn(Color) -> u8| {
        let around: u32 = neighbors.iter().map(|&c| u32::from(get(c))).sum();
        ((3 * around + 4 * u32::from(get(center))) / 16) as u8
    };
    Color::rgb(channel(Color::r), channel(Color::g), channel(Color::b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soften_keeps_flat_color() {
        let color = Color::rgb(200, 100, 50);
        assert_eq!(soften_pixel(color, [color; 4]), color);
    }

    #[test]
    fn identity_field_leaves_the_image_alone() {
        let size = Size::new(4, 3);
        let mut source = Canvas::new(size, Color::BLACK);
        source.set(Point::new(2, 1), Color::rgb(9, 9, 9));
        let mut target = Canvas::new(size, Color::BLACK);
        let field = WarpField::identity(size);
        remap(&source, &mut target, &field, &field, 0.5);
        assert_eq!(target.pixels(), source.pixels());
    }
}
