//! The effects the scheduler runs. Each one is randomized when the scheduler picks it, then
//! renders every frame until its time is up.

mod polygram;
mod ribbon;
mod surge;
mod twin_orbs;
mod undertow;

pub(crate) use polygram::Polygram;
pub(crate) use ribbon::Ribbon;
pub(crate) use surge::Surge;
pub(crate) use twin_orbs::TwinOrbs;
pub(crate) use undertow::Undertow;

use crate::audio_frame::AudioFrame;
use crate::frame::Frame;
use crate::geometry::Size;
use crate::levels::Levels;
use crate::rng::Rng;

pub(crate) trait Effect: Send {
    /// Picks fresh random settings. Called each time the scheduler starts the effect.
    fn randomize(&mut self, rng: &mut Rng, size: Size);

    /// Draws one frame.
    fn render(&mut self, frame: &mut Frame, input: &EffectInput<'_>, rng: &mut Rng);
}

/// What an effect gets to work with each frame, besides the frame itself.
pub(crate) struct EffectInput<'a> {
    pub audio: &'a AudioFrame,
    pub levels: Levels,
    pub fade: Fade,
}

/// Where an effect is in its lifetime.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Fade {
    /// Just started; `progress` goes from 0 to 1.
    In {
        progress: f64,
    },
    Steady,
    /// About to end; `progress` goes from 0 to 1.
    Out {
        progress: f64,
    },
}
