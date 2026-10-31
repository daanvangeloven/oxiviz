use std::f64::consts::FRAC_PI_2;
use std::ops::RangeInclusive;

use crate::rng::Rng;

/// A transition moves from the old warp to the new one in this many steps.
const STEPS: i32 = 23;
/// Draws when picking the pace, biasing toward [`Pace::Eased`].
const PACE_DRAWS: u32 = 3;
/// One in this many transitions draws its speed from [`WIDE_FRAMES_PER_STEP`].
const WIDE_SPEED_ONE_IN: u32 = 3;
const WIDE_FRAMES_PER_STEP: RangeInclusive<i32> = 1..=15;
const USUAL_FRAMES_PER_STEP: RangeInclusive<i32> = 10..=14;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Pace {
    /// Fast at first, slowing down toward the end.
    Eased,
    /// One step every `frames_per_step` frames.
    Regular,
    /// Each frame, a one in `frames_per_step` chance of a step.
    Random,
}

/// How far along the change from one warp to the next is.
#[derive(Clone, Debug)]
pub(super) struct Transition {
    pace: Pace,
    frames_per_step: i32,
    frame: i32,
    step: i32,
}

impl Transition {
    pub fn random(rng: &mut Rng) -> Self {
        let pace = match rng.int_biased_low(0..=2, PACE_DRAWS) {
            0 => Pace::Eased,
            1 => Pace::Regular,
            _ => Pace::Random,
        };
        let speeds = if rng.one_in(WIDE_SPEED_ONE_IN) { WIDE_FRAMES_PER_STEP } else { USUAL_FRAMES_PER_STEP };
        Self { pace, frames_per_step: rng.int(speeds), frame: 0, step: 0 }
    }

    /// A transition that has already arrived.
    pub fn finished() -> Self {
        Self { pace: Pace::Regular, frames_per_step: 1, frame: 0, step: STEPS }
    }

    pub fn advance(&mut self, rng: &mut Rng) {
        if self.step >= STEPS {
            return;
        }
        self.frame += 1;
        self.step = match self.pace {
            Pace::Eased => {
                // Covers all but the last step on a quarter sine, then jumps to the end.
                let last = STEPS - 1;
                let duration = self.frames_per_step * last;
                if self.frame >= duration {
                    STEPS
                } else {
                    let t = f64::from(self.frame) / f64::from(duration);
                    (f64::from(last) * (t * FRAC_PI_2).sin()) as i32
                }
            }
            Pace::Regular => self.step + i32::from(self.frame % self.frames_per_step == 0),
            Pace::Random => self.step + i32::from(rng.chance(1.0 / f64::from(self.frames_per_step))),
        };
    }

    /// `0` = all old warp, `1` = all new warp.
    pub fn progress(&self) -> f64 {
        f64::from(self.step.min(STEPS)) / f64::from(STEPS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pace_arrives() {
        let mut rng = Rng::new(5);
        for pace in [Pace::Eased, Pace::Regular, Pace::Random] {
            let mut transition = Transition { pace, frames_per_step: 3, frame: 0, step: 0 };
            for _ in 0..10_000 {
                transition.advance(&mut rng);
            }
            assert_eq!(transition.progress(), 1.0, "{pace:?}");
        }
    }

    #[test]
    fn progress_never_goes_back() {
        let mut rng = Rng::new(6);
        let mut transition = Transition { pace: Pace::Eased, frames_per_step: 2, frame: 0, step: 0 };
        let mut last = 0.0;
        for _ in 0..100 {
            transition.advance(&mut rng);
            assert!(transition.progress() >= last);
            last = transition.progress();
        }
    }
}
