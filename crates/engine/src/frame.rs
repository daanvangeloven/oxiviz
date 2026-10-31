use std::mem;

use crate::canvas::Canvas;
use crate::color::Color;
use crate::geometry::Size;

/// What effects draw into: the current image, the previous one for feedback passes, and how
/// much of the image will be shown.
#[derive(Debug)]
pub(crate) struct Frame {
    current: Canvas,
    previous: Canvas,
    background: Color,
    view_scale: f64,
}

impl Frame {
    pub fn new(size: Size, background: Color) -> Self {
        Self {
            current: Canvas::new(size, background),
            previous: Canvas::new(size, background),
            background,
            view_scale: 1.0,
        }
    }

    pub fn size(&self) -> Size {
        self.current.size()
    }

    pub fn background(&self) -> Color {
        self.background
    }

    pub fn current(&self) -> &Canvas {
        &self.current
    }

    pub fn current_mut(&mut self) -> &mut Canvas {
        &mut self.current
    }

    /// Starts a full-screen pass: the current image becomes the source, and the returned
    /// destination is the buffer that will become the current image.
    pub fn swap(&mut self) -> (&Canvas, &mut Canvas) {
        mem::swap(&mut self.current, &mut self.previous);
        (&self.previous, &mut self.current)
    }

    /// Fraction of the image shown, centered and stretched to fill the frame (1 = all of it).
    pub fn view_scale(&self) -> f64 {
        self.view_scale
    }

    pub fn set_view_scale(&mut self, scale: f64) {
        self.view_scale = scale;
    }

    /// Resets per-frame state; called before the effects render.
    pub fn begin(&mut self) {
        self.view_scale = 1.0;
    }
}
