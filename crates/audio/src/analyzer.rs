use std::f32::consts::TAU;
use std::sync::Arc;

use oxiviz_engine::{AudioFrame, Channel, SPECTRUM_BINS, WAVEFORM_SAMPLES};
use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};

use crate::decode::{Pcm, SAMPLE_RATE};

/// Samples per FFT. Twice the bin count, since the upper half of an FFT mirrors the lower.
const FFT_SIZE: usize = 2 * SPECTRUM_BINS;

/// Spectrum bytes are decibels: a full-scale sine (0 dB) is 255, and this many dB below it is 0.
///
/// This is tuned so the engine's bass level sits
/// around 0.3–0.8 on typical music, which is what its beat thresholds expect. Decibels keep
/// quiet and loud masters in that range, where a linear scale differs 10× between songs.
const SPECTRUM_FLOOR_DB: f32 = -60.0;

const BYTE_MAX: f32 = 255.0;
/// Waveform byte for silence, and how far a full-scale sample swings from it.
const WAVEFORM_ZERO: f32 = 128.0;
const WAVEFORM_SWING: f32 = 127.0;

/// Cuts decoded audio into one [`AudioFrame`] per video frame.
pub struct FrameAnalyzer {
    pcm: Pcm,
    fps: f64,
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    /// Turns FFT magnitudes into the amplitude of the sine that would produce them.
    normalization: f32,
    buffer: Vec<Complex32>,
    scratch: Vec<Complex32>,
}

impl FrameAnalyzer {
    pub fn new(pcm: Pcm, fps: f64) -> Self {
        let fft = FftPlanner::new().plan_fft_forward(FFT_SIZE);
        let window = hann_window(FFT_SIZE);
        let normalization = 2.0 / window.iter().sum::<f32>();
        let scratch = vec![Complex32::default(); fft.get_inplace_scratch_len()];
        Self { pcm, fps, fft, window, normalization, buffer: vec![Complex32::default(); FFT_SIZE], scratch }
    }

    /// Number of video frames the audio lasts.
    pub fn frame_count(&self) -> usize {
        (self.pcm.duration_secs() * self.fps).ceil() as usize
    }

    /// Fills `out` with the audio at video frame `index`.
    pub fn analyze(&mut self, index: usize, out: &mut AudioFrame) {
        let first_sample = (index as f64 / self.fps * f64::from(SAMPLE_RATE)) as usize;
        for channel in [Channel::Left, Channel::Right] {
            self.spectrum(channel, first_sample, &mut out.spectrum[channel as usize]);
            waveform(self.pcm.channel(channel), first_sample, &mut out.waveform[channel as usize]);
        }
    }

    /// The spectrum of a window centered on `center`. Samples before the start or past the end
    /// count as silence.
    fn spectrum(&mut self, channel: Channel, center: usize, out: &mut [u8; SPECTRUM_BINS]) {
        let samples = self.pcm.channel(channel);
        let start = center as isize - (FFT_SIZE / 2) as isize;
        for (i, (slot, weight)) in self.buffer.iter_mut().zip(&self.window).enumerate() {
            let sample = usize::try_from(start + i as isize).ok().and_then(|at| samples.get(at));
            *slot = Complex32::new(sample.copied().unwrap_or(0.0) * weight, 0.0);
        }
        self.fft.process_with_scratch(&mut self.buffer, &mut self.scratch);

        for (byte, bin) in out.iter_mut().zip(&self.buffer) {
            *byte = spectrum_byte(bin.norm() * self.normalization);
        }
    }
}

/// Maps a magnitude (1.0 = a full-scale sine) onto `0..=255` on a decibel scale.
fn spectrum_byte(magnitude: f32) -> u8 {
    let decibels = 20.0 * magnitude.max(f32::MIN_POSITIVE).log10();
    let fraction = 1.0 - decibels / SPECTRUM_FLOOR_DB;
    (fraction * BYTE_MAX).clamp(0.0, BYTE_MAX) as u8
}

/// The samples starting at `start`, as bytes around 128. Past the end counts as silence.
fn waveform(samples: &[f32], start: usize, out: &mut [u8; WAVEFORM_SAMPLES]) {
    for (i, byte) in out.iter_mut().enumerate() {
        let sample = samples.get(start + i).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        *byte = (WAVEFORM_ZERO + sample * WAVEFORM_SWING).round() as u8;
    }
}

fn hann_window(len: usize) -> Vec<f32> {
    (0..len).map(|i| 0.5 - 0.5 * (TAU * i as f32 / len as f32).cos()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One second of a sine at `frequency` Hz with peak `amplitude`, on both channels.
    fn sine(frequency: f32, amplitude: f32) -> Pcm {
        let samples: Vec<f32> = (0..SAMPLE_RATE)
            .map(|i| amplitude * (TAU * frequency * i as f32 / SAMPLE_RATE as f32).sin())
            .collect();
        Pcm { left: samples.clone(), right: samples }
    }

    #[test]
    fn sine_peaks_in_its_bin_at_its_amplitude() {
        let bin_width = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let bin = 100;
        let mut analyzer = FrameAnalyzer::new(sine(bin as f32 * bin_width, 0.5), 30.0);
        let mut frame = AudioFrame::silent();
        analyzer.analyze(15, &mut frame);

        let spectrum = &frame.spectrum[0];
        let peak = (0..SPECTRUM_BINS).max_by_key(|&i| spectrum[i]).unwrap();
        assert_eq!(peak, bin);
        assert_eq!(spectrum[bin], spectrum_byte(0.5), "a full bin measures the sine's amplitude");
    }

    #[test]
    fn spectrum_bytes_span_the_decibel_range() {
        assert_eq!(spectrum_byte(1.0), 255);
        assert_eq!(spectrum_byte(0.0), 0);
        assert_eq!(spectrum_byte(10f32.powf(SPECTRUM_FLOOR_DB / 40.0)), 127, "half the range");
    }

    #[test]
    fn silence_past_the_end() {
        let mut analyzer = FrameAnalyzer::new(sine(440.0, 0.5), 30.0);
        let mut frame = AudioFrame::silent();
        analyzer.analyze(1000, &mut frame);
        assert!(frame.spectrum[0].iter().all(|&b| b == 0));
        assert!(frame.waveform[0].iter().all(|&b| b == 128));
    }

    #[test]
    fn frame_count_covers_the_audio() {
        assert_eq!(FrameAnalyzer::new(sine(440.0, 0.5), 30.0).frame_count(), 30);
    }
}
