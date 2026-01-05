//! Parametric EQ Effect
//!
//! A multi-band parametric EQ for master output shaping.
//! Uses the BiquadEQ module for each band.

use crate::dsp::filters::{EQBand, EQBandType};

/// Parametric EQ with Low Shelf, Mid Peak, and High Shelf bands
///
/// A simple 3-band EQ suitable for master output processing.
pub struct ParametricEQ {
    /// Low band (shelf)
    low_band: EQBand,
    /// Mid band (peaking)
    mid_band: EQBand,
    /// High band (shelf)
    high_band: EQBand,

    /// Sample rate
    sample_rate: f32,
}

impl ParametricEQ {
    /// Create a new parametric EQ
    pub fn new(sample_rate: f32) -> Self {
        let mut low_band = EQBand::new(sample_rate);
        low_band.set_band_type(EQBandType::LowShelf);
        low_band.set_freq(200.0);
        low_band.set_q(0.707);
        low_band.set_gain(0.0);

        let mut mid_band = EQBand::new(sample_rate);
        mid_band.set_band_type(EQBandType::Peaking);
        mid_band.set_freq(1000.0);
        mid_band.set_q(1.0);
        mid_band.set_gain(0.0);

        let mut high_band = EQBand::new(sample_rate);
        high_band.set_band_type(EQBandType::HighShelf);
        high_band.set_freq(4000.0);
        high_band.set_q(0.707);
        high_band.set_gain(0.0);

        Self {
            low_band,
            mid_band,
            high_band,
            sample_rate,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.low_band.set_sample_rate(sample_rate);
        self.mid_band.set_sample_rate(sample_rate);
        self.high_band.set_sample_rate(sample_rate);
    }

    /// Set low band frequency
    pub fn set_low_freq(&mut self, freq: f32) {
        self.low_band.set_freq(freq);
    }

    /// Set low band gain in dB
    pub fn set_low_gain(&mut self, gain: f32) {
        self.low_band.set_gain(gain);
    }

    /// Set mid band frequency
    pub fn set_mid_freq(&mut self, freq: f32) {
        self.mid_band.set_freq(freq);
    }

    /// Set mid band Q
    pub fn set_mid_q(&mut self, q: f32) {
        self.mid_band.set_q(q);
    }

    /// Set mid band gain in dB
    pub fn set_mid_gain(&mut self, gain: f32) {
        self.mid_band.set_gain(gain);
    }

    /// Set high band frequency
    pub fn set_high_freq(&mut self, freq: f32) {
        self.high_band.set_freq(freq);
    }

    /// Set high band gain in dB
    pub fn set_high_gain(&mut self, gain: f32) {
        self.high_band.set_gain(gain);
    }

    /// Reset all bands
    pub fn reset(&mut self) {
        self.low_band.reset();
        self.mid_band.reset();
        self.high_band.reset();
    }

    /// Process one sample
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let low_out = self.low_band.process(input);
        let mid_out = self.mid_band.process(low_out);
        self.high_band.process(mid_out)
    }
}

impl Default for ParametricEQ {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

/// Flexible N-band parametric EQ
///
/// Allows creating an EQ with any number of bands.
pub struct MultibandEQ<const N: usize> {
    /// EQ bands
    bands: [EQBand; N],

    /// Sample rate
    sample_rate: f32,
}

impl<const N: usize> MultibandEQ<N> {
    /// Create a new multiband EQ
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bands: core::array::from_fn(|_| EQBand::new(sample_rate)),
            sample_rate,
        }
    }

    /// Set sample rate for all bands
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for band in &mut self.bands {
            band.set_sample_rate(sample_rate);
        }
    }

    /// Get mutable reference to a specific band
    pub fn band_mut(&mut self, index: usize) -> Option<&mut EQBand> {
        self.bands.get_mut(index)
    }

    /// Get reference to a specific band
    pub fn band(&self, index: usize) -> Option<&EQBand> {
        self.bands.get(index)
    }

    /// Reset all bands
    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }

    /// Process one sample through all bands in series
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = input;
        for band in &mut self.bands {
            output = band.process(output);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parametric_eq_basic() {
        let mut eq = ParametricEQ::new(44100.0);
        eq.set_low_gain(3.0);
        eq.set_mid_gain(-2.0);
        eq.set_high_gain(1.0);

        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let output = eq.process(input);
            assert!(output.is_finite(), "ParametricEQ produced invalid output");
        }
    }

    #[test]
    fn test_parametric_eq_frequencies() {
        let mut eq = ParametricEQ::new(44100.0);
        eq.set_low_freq(100.0);
        eq.set_mid_freq(2000.0);
        eq.set_high_freq(8000.0);

        eq.set_low_gain(6.0);
        eq.set_mid_gain(6.0);
        eq.set_high_gain(6.0);

        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let output = eq.process(input);
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_multiband_eq() {
        let mut eq = MultibandEQ::<5>::new(44100.0);

        // Configure bands
        if let Some(band) = eq.band_mut(0) {
            band.set_band_type(EQBandType::HighPass);
            band.set_freq(40.0);
            band.set_q(0.707);
        }
        if let Some(band) = eq.band_mut(1) {
            band.set_band_type(EQBandType::LowShelf);
            band.set_freq(100.0);
            band.set_gain(2.0);
        }
        if let Some(band) = eq.band_mut(2) {
            band.set_band_type(EQBandType::Peaking);
            band.set_freq(1000.0);
            band.set_q(2.0);
            band.set_gain(-3.0);
        }
        if let Some(band) = eq.band_mut(3) {
            band.set_band_type(EQBandType::HighShelf);
            band.set_freq(8000.0);
            band.set_gain(1.5);
        }
        if let Some(band) = eq.band_mut(4) {
            band.set_band_type(EQBandType::LowPass);
            band.set_freq(18000.0);
            band.set_q(0.707);
        }

        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let output = eq.process(input);
            assert!(output.is_finite(), "MultibandEQ produced invalid output");
        }
    }

    #[test]
    fn test_flat_eq() {
        let mut eq = ParametricEQ::new(44100.0);
        // All gains at 0 should be unity

        // Let it settle
        for _ in 0..100 {
            let _ = eq.process(0.5);
        }

        // Should pass through approximately unchanged
        let input = 0.5;
        let output = eq.process(input);
        assert!(
            (output - input).abs() < 0.15,
            "Flat EQ should pass signal, got {} vs {}",
            output,
            input
        );
    }
}
