//! VA (Virtual Analog) One-Pole Filter
//!
//! Based on Will Pirkle's "Designing Software Synthesizer Plug-Ins in C++"
//! This filter is used as a building block for the ladder filter.

use crate::dsp::math::fast_tan;
use crate::constants::TWO_PI;

/// Virtual Analog One-Pole Filter
///
/// A first-order lowpass or highpass filter using the bilinear transform
/// with frequency prewarping for accurate cutoff frequency.
#[derive(Debug, Clone)]
pub struct VAOnePoleFilter {
    /// Feed-forward coefficient (alpha)
    pub alpha: f32,

    /// Feedback coefficient (beta) for ladder filter integration
    pub beta: f32,

    /// Pre-gain (gamma)
    gamma: f32,

    /// Feedback input coefficient (delta)
    delta: f32,

    /// Feedback output scalar (epsilon)
    epsilon: f32,

    /// Input gain (a_0)
    a_0: f32,

    /// Feedback value from external source
    pub feedback: f32,

    /// Delay element (z^-1)
    z_1: f32,

    /// Whether this is a lowpass (true) or highpass (false) filter
    is_lowpass: bool,

    /// Sample rate
    sample_rate: f32,

    /// One over sample rate
    one_over_sample_rate: f32,
}

impl VAOnePoleFilter {
    /// Create a new VA one-pole filter
    pub fn new(sample_rate: f32) -> Self {
        Self {
            alpha: 1.0,
            beta: 0.0,
            gamma: 1.0,
            delta: 0.0,
            epsilon: 0.0,
            a_0: 1.0,
            feedback: 0.0,
            z_1: 0.0,
            is_lowpass: true,
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
        }
    }

    /// Set to lowpass mode
    pub fn set_lowpass(&mut self) {
        self.is_lowpass = true;
    }

    /// Set to highpass mode
    pub fn set_highpass(&mut self) {
        self.is_lowpass = false;
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.z_1 = 0.0;
        self.feedback = 0.0;
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
    }

    /// Update coefficients for a given cutoff frequency
    pub fn update(&mut self, cutoff: f32) {
        // Prewarp for BZT (Bilinear Z-Transform)
        let wd = TWO_PI * cutoff;
        let wa = 2.0 * self.sample_rate * fast_tan(wd * self.one_over_sample_rate * 0.5);
        let g = wa * self.one_over_sample_rate * 0.5;

        self.alpha = g / (1.0 + g);
    }

    /// Set the frequency directly (convenience method)
    pub fn set_frequency(&mut self, freq: f32) {
        self.update(freq);
    }

    // Setters for diode/ladder filter integration

    /// Set alpha coefficient directly
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    /// Set beta coefficient directly
    pub fn set_beta(&mut self, beta: f32) {
        self.beta = beta;
    }

    /// Set gamma coefficient directly
    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma;
    }

    /// Set delta coefficient directly
    pub fn set_delta(&mut self, delta: f32) {
        self.delta = delta;
    }

    /// Set epsilon coefficient directly
    pub fn set_epsilon(&mut self, epsilon: f32) {
        self.epsilon = epsilon;
    }

    /// Set a_0 coefficient directly
    pub fn set_a0(&mut self, a0: f32) {
        self.a_0 = a0;
    }

    /// Set feedback value
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    /// Get feedback output for ladder filter integration
    #[inline]
    pub fn get_feedback_output(&self) -> f32 {
        self.beta * (self.z_1 + self.feedback * self.delta)
    }

    /// Process one sample
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        // For diode filter support (external feedback)
        let xn = input * self.gamma + self.feedback + self.epsilon * self.get_feedback_output();

        // Calculate v(n)
        let vn = (self.a_0 * xn - self.z_1) * self.alpha;

        // Form LP output
        let lpf = vn + self.z_1;

        // Update memory
        self.z_1 = vn + lpf;

        if self.is_lowpass {
            lpf
        } else {
            xn - lpf
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_va_one_pole_lowpass() {
        let mut filter = VAOnePoleFilter::new(44100.0);
        filter.update(1000.0);
        filter.set_lowpass();

        // Process some samples
        let mut output = 0.0;
        for i in 0..1000 {
            // Input is a step function
            let input = if i < 100 { 0.0 } else { 1.0 };
            output = filter.process(input);
        }

        // Output should be close to 1.0 after settling
        assert!(output > 0.9 && output <= 1.0, "Output: {}", output);
    }

    #[test]
    fn test_va_one_pole_reset() {
        let mut filter = VAOnePoleFilter::new(44100.0);
        filter.update(1000.0);

        // Process some samples
        for _ in 0..100 {
            filter.process(1.0);
        }

        // Reset
        filter.reset();

        // z_1 should be 0
        assert_eq!(filter.z_1, 0.0);
    }
}
