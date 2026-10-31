//! Loudness and beats, measured from the spectrum.

use crate::audio_frame::{AudioFrame, Channel};

/// The bins summed into the bass level: the lowest few, about 20–130 Hz at 44.1 kHz.
const BASS_BINS: [(Channel, usize); 6] = [
    (Channel::Left, 1),
    (Channel::Left, 3),
    (Channel::Left, 5),
    (Channel::Right, 2),
    (Channel::Right, 4),
    (Channel::Right, 6),
];
const BASS_DIVISOR: f64 = 1200.0;

/// The bins summed into the treble level: around 10.8 kHz at 44.1 kHz.
const TREBLE_BINS: [(Channel, usize); 6] = [
    (Channel::Left, 501),
    (Channel::Left, 503),
    (Channel::Left, 505),
    (Channel::Right, 502),
    (Channel::Right, 504),
    (Channel::Right, 506),
];
const TREBLE_DIVISOR: f64 = 600.0;

/// Frames of bass history that the average is taken over.
const HISTORY_FRAMES: usize = 30;
/// A beat is bass this much louder than the average...
const BEAT_RATIO: f64 = 1.1;
/// ...at least this many frames after the previous beat.
const MIN_FRAMES_BETWEEN_BEATS: u32 = 10;
/// A big beat is a beat this much louder than the average...
const BIG_BEAT_RATIO: f64 = 1.2;
/// ...that's also at least this loud...
const BIG_BEAT_MIN_BASS: f64 = 0.9;
/// ...and at least this many frames after the previous big beat.
const MIN_FRAMES_BETWEEN_BIG_BEATS: u32 = 21;

/// What effects know about the music in the current frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Levels {
    /// Bass loudness, about `0..=1.27`. Effects scale sizes and speeds by it.
    pub bass: f64,
    /// Treble loudness, about `0..=2.55`.
    pub treble: f64,
    /// `bass` minus its recent average.
    pub bass_change: f64,
    pub beat: bool,
    pub big_beat: bool,
}

/// Turns each frame's spectrum into [`Levels`], keeping the history that beat detection needs.
#[derive(Clone, Debug)]
pub struct LevelTracker {
    history: [f64; HISTORY_FRAMES],
    next_slot: usize,
    frames_since_beat: u32,
    frames_since_big_beat: u32,
}

impl Default for LevelTracker {
    fn default() -> Self {
        Self { history: [0.0; HISTORY_FRAMES], next_slot: 0, frames_since_beat: 0, frames_since_big_beat: 0 }
    }
}

impl LevelTracker {
    pub fn update(&mut self, audio: &AudioFrame) -> Levels {
        let bass = sum_bins(audio, &BASS_BINS) / BASS_DIVISOR;
        self.history[self.next_slot] = bass;
        self.next_slot = (self.next_slot + 1) % HISTORY_FRAMES;
        let average = self.history.iter().sum::<f64>() / HISTORY_FRAMES as f64;

        self.frames_since_beat += 1;
        self.frames_since_big_beat += 1;
        let beat = self.frames_since_beat >= MIN_FRAMES_BETWEEN_BEATS && bass > average * BEAT_RATIO;
        let big_beat = beat
            && bass > BIG_BEAT_MIN_BASS
            && bass > average * BIG_BEAT_RATIO
            && self.frames_since_big_beat >= MIN_FRAMES_BETWEEN_BIG_BEATS;
        if beat {
            self.frames_since_beat = 0;
        }
        if big_beat {
            self.frames_since_big_beat = 0;
        }

        Levels {
            bass,
            treble: sum_bins(audio, &TREBLE_BINS) / TREBLE_DIVISOR,
            bass_change: bass - average,
            beat,
            big_beat,
        }
    }
}

fn sum_bins(audio: &AudioFrame, bins: &[(Channel, usize)]) -> f64 {
    bins.iter().map(|&(channel, bin)| f64::from(audio.bin(channel, bin))).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_with_bass(value: u8) -> AudioFrame {
        let mut audio = AudioFrame::silent();
        for (channel, bin) in BASS_BINS {
            audio.spectrum[channel as usize][bin] = value;
        }
        audio
    }

    #[test]
    fn bass_is_the_scaled_sum_of_the_bass_bins() {
        let levels = LevelTracker::default().update(&frame_with_bass(200));
        assert!((levels.bass - 1.0).abs() < 1e-9);
    }

    #[test]
    fn beats_need_a_jump_and_a_gap() {
        let mut tracker = LevelTracker::default();
        let quiet = frame_with_bass(50);
        let loud = frame_with_bass(220);
        // Fill the history, so the average is the quiet level.
        for _ in 0..HISTORY_FRAMES {
            tracker.update(&quiet);
        }
        assert!(!tracker.update(&quiet).beat, "steady bass is no beat");
        for _ in 0..MIN_FRAMES_BETWEEN_BIG_BEATS {
            tracker.update(&quiet);
        }
        let first = tracker.update(&loud);
        assert!(first.beat && first.big_beat);
        assert!(!tracker.update(&quiet).beat);
        assert!(!tracker.update(&loud).beat, "too soon after the last beat");
    }
}
