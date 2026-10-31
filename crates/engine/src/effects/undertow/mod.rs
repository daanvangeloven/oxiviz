//! The feedback warp behind the visualizer's look. Every frame copies the previous one through a warp
//! (a per-pixel "copy from there" map), then softens it, so whatever the overlays draw leaves
//! trails that swirl, scroll and mirror. A new warp phases in gradually.

mod field;
mod transition;
mod warps;

use field::WarpField;
use transition::Transition;
use warps::Warp;

use crate::effects::{Effect, EffectInput};
use crate::frame::Frame;
use crate::geometry::Size;
use crate::rng::Rng;

/// Rows of a new warp computed per frame, so it sweeps in from the top.
const ROWS_BUILT_PER_FRAME: usize = 3;
/// One in this many warps is blended with a second one.
const COMBINE_ONE_IN: u32 = 5;

pub(crate) struct Undertow {
    /// Where the transition starts: the previous warp.
    from: WarpField,
    /// Where it ends: the new warp, filled in a few rows per frame.
    to: WarpField,
    rows_built: usize,
    primary: Option<Box<dyn Warp>>,
    secondary: Option<Box<dyn Warp>>,
    transition: Transition,
}

impl Undertow {
    pub fn new() -> Self {
        let size = Size::new(0, 0);
        Self {
            from: WarpField::identity(size),
            to: WarpField::identity(size),
            rows_built: 0,
            primary: None,
            secondary: None,
            transition: Transition::finished(),
        }
    }

    fn build_rows(&mut self) {
        let Some(primary) = &self.primary else { return };
        let end = (self.rows_built + ROWS_BUILT_PER_FRAME).min(self.to.size().height);
        for y in self.rows_built..end {
            self.to.build_row(y, primary.as_ref(), self.secondary.as_deref());
        }
        self.rows_built = end;
    }
}

impl Effect for Undertow {
    fn randomize(&mut self, rng: &mut Rng, size: Size) {
        if self.to.size() != size {
            self.to = WarpField::identity(size);
        }
        self.from.clone_from(&self.to);

        let primary = warps::random(rng, size);
        self.secondary = if primary.combinable() && rng.one_in(COMBINE_ONE_IN) {
            Some(warps::random(rng, size)).filter(|warp| warp.combinable())
        } else {
            None
        };
        self.primary = Some(primary);
        self.rows_built = 0;
        self.transition = Transition::random(rng);
    }

    fn render(&mut self, frame: &mut Frame, _input: &EffectInput<'_>, rng: &mut Rng) {
        self.build_rows();
        self.transition.advance(rng);

        let (previous, current) = frame.swap();
        field::remap(previous, current, &self.from, &self.to, self.transition.progress());
        let (previous, current) = frame.swap();
        field::soften(previous, current);

        let background = frame.background();
        let canvas = frame.current_mut();
        let last_row = canvas.size().height - 1;
        canvas.fill_row(0, background);
        canvas.fill_row(last_row, background);
    }
}
