/// Number of spectrum bins per channel, spread evenly from 0 Hz to half the sample rate.
pub const SPECTRUM_BINS: usize = 1024;

/// Number of waveform samples per channel.
pub const WAVEFORM_SAMPLES: usize = 1024;

/// Waveform value for silence.
pub(crate) const WAVEFORM_ZERO: u8 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Left = 0,
    Right = 1,
}

/// The audio for one frame: a byte spectrum and waveform per channel.
#[derive(Clone, Debug)]
pub struct AudioFrame {
    /// Magnitude per bin, `0..=255`.
    pub spectrum: [[u8; SPECTRUM_BINS]; 2],
    /// Samples starting at the frame's time, with 128 as zero.
    pub waveform: [[u8; WAVEFORM_SAMPLES]; 2],
}

impl AudioFrame {
    pub fn silent() -> Self {
        Self { spectrum: [[0; SPECTRUM_BINS]; 2], waveform: [[WAVEFORM_ZERO; WAVEFORM_SAMPLES]; 2] }
    }

    pub(crate) fn bin(&self, channel: Channel, bin: usize) -> u8 {
        self.spectrum[channel as usize][bin]
    }

    /// Both channels mixed, as an offset from zero in `-1.0..=1.0`.
    pub(crate) fn mixed_sample(&self, index: usize) -> f64 {
        let index = index.min(WAVEFORM_SAMPLES - 1);
        let sum = u32::from(self.waveform[0][index]) + u32::from(self.waveform[1][index]);
        (f64::from(sum) / 2.0 - f64::from(WAVEFORM_ZERO)) / f64::from(WAVEFORM_ZERO)
    }
}

impl Default for AudioFrame {
    fn default() -> Self {
        Self::silent()
    }
}
