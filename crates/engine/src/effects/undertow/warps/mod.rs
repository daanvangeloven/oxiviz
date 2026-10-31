//! Warps: for each pixel, where in the previous frame to copy it from.

mod kaleido;
mod scroll;
mod swirl;
mod weave;

use kaleido::Kaleido;
use scroll::Scroll;
use swirl::Swirl;
use weave::Weave;

use crate::geometry::{Point, Size};
use crate::rng::Rng;

pub(super) trait Warp: Send {
    /// The pixel that `p` copies from. May be outside the frame.
    fn source(&self, p: Point) -> Point;

    /// Whether this warp may be blended with a second one.
    fn combinable(&self) -> bool {
        true
    }

    /// What a pixel copies when its source is outside the frame.
    fn outside(&self) -> Outside {
        Outside::StayPut
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Outside {
    /// The pixel copies itself.
    StayPut,
    /// The pixel copies the top-left corner. The top row is cleared every frame, so the
    /// background color flows in from the edges.
    TopLeft,
}

/// How often each warp is picked, out of [`TOTAL_WEIGHT`].
const KALEIDO_WEIGHT: u32 = 4;
const SCROLL_WEIGHT: u32 = 2;
const SWIRL_WEIGHT: u32 = 2;
const WEAVE_WEIGHT: u32 = 2;
const TOTAL_WEIGHT: u32 = KALEIDO_WEIGHT + SCROLL_WEIGHT + SWIRL_WEIGHT + WEAVE_WEIGHT;

/// A random warp with random settings.
pub(super) fn random(rng: &mut Rng, size: Size) -> Box<dyn Warp> {
    let pick = rng.next_u32() % TOTAL_WEIGHT;
    if pick < KALEIDO_WEIGHT {
        Box::new(Kaleido::random(rng, size))
    } else if pick < KALEIDO_WEIGHT + SCROLL_WEIGHT {
        Box::new(Scroll::random(rng, size))
    } else if pick < KALEIDO_WEIGHT + SCROLL_WEIGHT + SWIRL_WEIGHT {
        Box::new(Swirl::random(rng, size))
    } else {
        Box::new(Weave::random(rng, size))
    }
}
