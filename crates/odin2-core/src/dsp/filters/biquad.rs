//! Biquad Filter
//!
//! A general purpose biquad (bi-quadratic) filter using Direct Form II
//! transposed structure. Used as a building block for allpass, resonator,
//! and other filter types.

use crate::dsp::math::fast_cos;
use core::f32::consts::PI;

/// Biquad filter using Direct Form II transposed
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    /// Feed-forward coefficient b0
    pub b0: f32,
    /// Feed-forward coefficient b1
    pub b1: f32,
    /// Feed-forward coefficient b2
    pub b2: f32,
    /// Feedback coefficient a1
    pub a1: f32,
    /// Feedback coefficient a2
    pub a2: f32,

    /// Delay element z1
    z1: f32,
    /// Delay element z2
    z2: f32,

    /// Sample rate
    sample_rate: f32,
    /// One over sample rate
    one_over_sample_rate: f32,
}

impl BiquadFilter {
    /// Create a new biquad filter
    pub fn new(sample_rate: f32) -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get one over sample rate
    pub fn one_over_sample_rate(&self) -> f32 {
        self.one_over_sample_rate
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    /// Process one sample using Direct Form II transposed
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;
        output
    }
}

/// Biquad allpass filter for phaser effect
#[derive(Debug, Clone)]
pub struct BiquadAllpass {
    filter: BiquadFilter,
    radius: f32,
}

impl BiquadAllpass {
    /// Create a new biquad allpass filter
    pub fn new(sample_rate: f32) -> Self {
        Self {
            filter: BiquadFilter::new(sample_rate),
            radius: 1.32,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.filter.set_sample_rate(sample_rate);
    }

    /// Set the radius (controls bandwidth/resonance)
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    /// Set the center frequency
    pub fn set_frequency(&mut self, freq: f32) {
        // Clamp frequency to valid range
        let freq = freq.clamp(20.0, self.filter.sample_rate() * 0.49);
        let w0 = 2.0 * PI * freq * self.filter.one_over_sample_rate();

        // Second-order allpass filter coefficients
        // Using standard biquad allpass design
        let cos_w0 = fast_cos(w0);
        let alpha = libm::sinf(w0) / (2.0 * self.radius);

        // Normalize by a0
        let a0 = 1.0 + alpha;
        let a0_inv = 1.0 / a0;

        self.filter.b0 = (1.0 - alpha) * a0_inv;
        self.filter.b1 = -2.0 * cos_w0 * a0_inv;
        self.filter.b2 = (1.0 + alpha) * a0_inv;
        self.filter.a1 = -2.0 * cos_w0 * a0_inv;
        self.filter.a2 = (1.0 - alpha) * a0_inv;
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.filter.reset();
    }

    /// Process one sample
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        self.filter.process(input)
    }
}

/// Biquad resonator for formant filter
#[derive(Debug, Clone)]
pub struct BiquadResonator {
    filter: BiquadFilter,
    radius: f32,
    freq: f32,
}

/// Gain factor for resonator
const RESONATOR_GAIN_FACTOR: f32 = 0.08;

impl BiquadResonator {
    /// Create a new biquad resonator
    pub fn new(sample_rate: f32) -> Self {
        let mut filter = BiquadFilter::new(sample_rate);
        // Initial resonator coefficients with zeros at 0 and Nyquist
        filter.b0 = RESONATOR_GAIN_FACTOR;
        filter.b1 = 0.0;
        filter.b2 = -RESONATOR_GAIN_FACTOR;

        Self {
            filter,
            radius: 0.996,
            freq: 2000.0,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.filter.set_sample_rate(sample_rate);
        self.recalculate_coefficients();
    }

    /// Set the radius (controls bandwidth/Q)
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.recalculate_coefficients();
    }

    /// Set the center frequency
    pub fn set_frequency(&mut self, freq: f32) {
        self.freq = freq;
        self.recalculate_coefficients();
    }

    /// Recalculate filter coefficients
    fn recalculate_coefficients(&mut self) {
        let w0 = 2.0 * PI * self.freq * self.filter.one_over_sample_rate();
        let cos_w0 = fast_cos(w0);

        self.filter.a1 = -2.0 * self.radius * cos_w0;
        self.filter.a2 = self.radius * self.radius;
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.filter.reset();
    }

    /// Process one sample
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        self.filter.process(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biquad_basic() {
        let mut filter = BiquadFilter::new(44100.0);

        // Process some samples
        for i in 0..1000 {
            let input = if i % 100 < 50 { 1.0 } else { -1.0 };
            let output = filter.process(input);
            assert!(output.is_finite(), "Filter produced invalid output");
        }
    }

    #[test]
    fn test_biquad_allpass() {
        let mut filter = BiquadAllpass::new(44100.0);
        filter.set_frequency(1000.0);

        // Process some samples
        for i in 0..1000 {
            let input = if i % 100 < 50 { 1.0 } else { -1.0 };
            let output = filter.process(input);
            assert!(output.is_finite(), "Allpass produced invalid output");
        }
    }

    #[test]
    fn test_biquad_resonator() {
        let mut filter = BiquadResonator::new(44100.0);
        filter.set_frequency(1000.0);

        // Process some samples
        for i in 0..1000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let output = filter.process(input);
            assert!(output.is_finite(), "Resonator produced invalid output");
        }
    }
}
