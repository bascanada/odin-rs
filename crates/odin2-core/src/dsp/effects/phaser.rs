//! Phaser Effect
//!
//! A stereo phaser effect using 12 biquad allpass filters per channel.
//! Creates the classic swooshing phaser sound through phase cancellation.

use crate::dsp::filters::BiquadAllpass;

/// Maximum LFO amplitude for frequency modulation
const PHASER_MAX_LFO_AMPLITUDE: f32 = 4000.0;

/// Number of allpass stages per channel
const NUM_STAGES: usize = 12;

/// Phaser effect
pub struct Phaser {
    /// Left channel allpass filters
    allpass_left: [BiquadAllpass; NUM_STAGES],
    /// Right channel allpass filters
    allpass_right: [BiquadAllpass; NUM_STAGES],

    /// Sample rate
    sample_rate: f32,

    /// LFO position for left channel (0.0 to 1.0)
    lfo_pos_left: f32,
    /// LFO position for right channel (90 degrees offset)
    lfo_pos_right: f32,
    /// LFO increment per sample
    lfo_inc: f32,
    /// LFO frequency in Hz
    lfo_freq: f32,
    /// LFO sign for left channel
    lfo_sign_left: i8,
    /// LFO sign for right channel
    lfo_sign_right: i8,

    /// Base frequency
    base_freq: f32,
    /// LFO modulation amount (0.0 to 1.5)
    amount: f32,
    /// Dry/wet mix (0.0 to 0.5)
    dry_wet: f32,
    /// Feedback amount (0.0 to 0.97)
    feedback: f32,
    /// Filter radius (bandwidth)
    radius: f32,
    /// Width parameter for stereo spread
    width: f32,

    /// Stored output for feedback (left)
    store_output_left: f32,
    /// Stored output for feedback (right)
    store_output_right: f32,

    /// Sync time numerator
    sync_numerator: f32,
    /// Sync time denominator
    sync_denominator: f32,
    /// Sync time ratio
    sync_ratio: f32,
}

impl Phaser {
    /// Create a new phaser effect
    pub fn new(sample_rate: f32) -> Self {
        let allpass_left = core::array::from_fn(|_| BiquadAllpass::new(sample_rate));
        let allpass_right = core::array::from_fn(|_| BiquadAllpass::new(sample_rate));

        let mut phaser = Self {
            allpass_left,
            allpass_right,
            sample_rate,
            lfo_pos_left: 0.0,
            lfo_pos_right: 0.5, // 90 degrees offset
            lfo_inc: 0.0,
            lfo_freq: 0.25,
            lfo_sign_left: 1,
            lfo_sign_right: 1,
            base_freq: 4000.0,
            amount: 0.3,
            dry_wet: 0.25, // Stored as 0-0.5 internally
            feedback: 0.25 * 0.97,
            radius: 1.32,
            width: 1.0,
            store_output_left: 0.0,
            store_output_right: 0.0,
            sync_numerator: 3.0,
            sync_denominator: 16.0,
            sync_ratio: 3.0 / 16.0,
        };

        phaser.set_lfo_freq(0.25);
        phaser.set_radius(1.32);
        phaser
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.set_lfo_freq(self.lfo_freq);

        for i in 0..NUM_STAGES {
            self.allpass_left[i].set_sample_rate(sample_rate);
            self.allpass_right[i].set_sample_rate(sample_rate);
        }
    }

    /// Set LFO frequency in Hz
    pub fn set_lfo_freq(&mut self, freq: f32) {
        self.lfo_freq = freq.clamp(0.01, 20.0);
        self.lfo_inc = 2.0 * self.lfo_freq / self.sample_rate;
    }

    /// Set LFO frequency from BPM
    pub fn set_lfo_from_bpm(&mut self, bpm: f32) {
        let freq = bpm / self.sync_ratio / 240.0;
        self.set_lfo_freq(freq);
    }

    /// Set sync time ratio
    pub fn set_sync_ratio(&mut self, numerator: f32, denominator: f32) {
        self.sync_numerator = numerator;
        self.sync_denominator = denominator;
        self.sync_ratio = numerator / denominator;
    }

    /// Set base frequency
    pub fn set_base_freq(&mut self, freq: f32) {
        self.base_freq = freq.clamp(400.0, 8000.0);
    }

    /// Set modulation amount (0.0 to 1.0)
    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.5);
    }

    /// Set dry/wet mix (0.0 to 1.0)
    pub fn set_dry_wet(&mut self, mix: f32) {
        self.dry_wet = mix.clamp(0.0, 1.0) * 0.5;
    }

    /// Set feedback (0.0 to 1.0)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 1.0) * 0.97;
    }

    /// Set filter radius (controls resonance/bandwidth)
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius.clamp(0.5, 2.0);
        for i in 0..NUM_STAGES {
            self.allpass_left[i].set_radius(self.radius);
            self.allpass_right[i].set_radius(self.radius);
        }
    }

    /// Reset the phaser
    pub fn reset(&mut self) {
        for i in 0..NUM_STAGES {
            self.allpass_left[i].reset();
            self.allpass_right[i].reset();
        }
        self.lfo_pos_left = 0.0;
        self.lfo_pos_right = 0.5;
        self.lfo_sign_left = 1;
        self.lfo_sign_right = 1;
        self.store_output_left = 0.0;
        self.store_output_right = 0.0;
    }

    /// Reset LFO position
    pub fn reset_lfo(&mut self) {
        self.lfo_pos_left = 0.0;
        self.lfo_pos_right = 0.5;
        self.lfo_sign_left = 1;
        self.lfo_sign_right = 1;
    }

    /// Cheap sine approximation using parabola
    fn lfo_sine(pos: f32, sign: i8) -> f32 {
        4.0 * pos * (1.0 - pos) * sign as f32
    }

    /// Increment left LFO
    fn increment_lfo_left(&mut self) {
        self.lfo_pos_left += self.lfo_inc;
        while self.lfo_pos_left > 1.0 {
            self.lfo_pos_left -= 1.0;
            self.lfo_sign_left *= -1;
        }
    }

    /// Increment right LFO
    fn increment_lfo_right(&mut self) {
        self.lfo_pos_right += self.lfo_inc;
        while self.lfo_pos_right > 1.0 {
            self.lfo_pos_right -= 1.0;
            self.lfo_sign_right *= -1;
        }
    }

    /// Set frequencies for left channel allpass filters
    fn set_frequency_left(&mut self, freq: f32) {
        let w = self.width;
        let offsets = [
            1.0 - 0.4 * w,
            1.0,
            1.0 + 0.2 * w,
            1.0 + 0.4 * w,
            1.0 - 0.13 * w,
            1.0 - 0.34 * w,
            1.0 - 0.375 * w,
            1.0 - 0.23 * w,
            1.0 + 0.13 * w,
            1.0 + 0.36 * w,
            1.0 - 0.09 * w,
            1.0 - 0.29 * w,
        ];
        for (i, &offset) in offsets.iter().enumerate() {
            self.allpass_left[i].set_frequency(freq * offset);
        }
    }

    /// Set frequencies for right channel allpass filters
    fn set_frequency_right(&mut self, freq: f32) {
        let w = self.width;
        let offsets = [
            1.0 - 0.4 * w,
            1.0,
            1.0 + 0.2 * w,
            1.0 + 0.4 * w,
            1.0 - 0.13 * w,
            1.0 - 0.34 * w,
            1.0 - 0.375 * w,
            1.0 - 0.23 * w,
            1.0 + 0.13 * w,
            1.0 + 0.36 * w,
            1.0 - 0.09 * w,
            1.0 - 0.29 * w,
        ];
        for (i, &offset) in offsets.iter().enumerate() {
            self.allpass_right[i].set_frequency(freq * offset);
        }
    }

    /// Process stereo input
    /// Returns (left_out, right_out)
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let left_out = self.process_left(left_in);
        let right_out = self.process_right(right_in);
        (left_out, right_out)
    }

    /// Process left channel
    fn process_left(&mut self, input: f32) -> f32 {
        // Apply feedback
        let feedback_clamped = self.feedback.clamp(0.0, 0.95);
        let input_with_feedback = input + self.store_output_left * feedback_clamped;

        // Get LFO value
        self.increment_lfo_left();
        let lfo = Self::lfo_sine(self.lfo_pos_left, self.lfo_sign_left);

        // Calculate modulated frequency
        let freq = self.base_freq + lfo * self.amount * PHASER_MAX_LFO_AMPLITUDE;
        let freq = freq.clamp(200.0, 10000.0);
        self.set_frequency_left(freq);

        // Process through allpass chain
        let mut phase_shifted = input_with_feedback;
        for i in 0..NUM_STAGES {
            phase_shifted = self.allpass_left[i].process(phase_shifted);
        }

        // Store for feedback
        self.store_output_left = phase_shifted;

        // Mix dry and wet
        (1.0 - self.dry_wet) * input + self.dry_wet * phase_shifted
    }

    /// Process right channel
    fn process_right(&mut self, input: f32) -> f32 {
        // Apply feedback
        let feedback_clamped = self.feedback.clamp(0.0, 0.95);
        let input_with_feedback = input + self.store_output_right * feedback_clamped;

        // Get LFO value
        self.increment_lfo_right();
        let lfo = Self::lfo_sine(self.lfo_pos_right, self.lfo_sign_right);

        // Calculate modulated frequency
        let freq = self.base_freq + lfo * self.amount * PHASER_MAX_LFO_AMPLITUDE;
        let freq = freq.clamp(200.0, 10000.0);
        self.set_frequency_right(freq);

        // Process through allpass chain
        let mut phase_shifted = input_with_feedback;
        for i in 0..NUM_STAGES {
            phase_shifted = self.allpass_right[i].process(phase_shifted);
        }

        // Store for feedback
        self.store_output_right = phase_shifted;

        // Mix dry and wet
        (1.0 - self.dry_wet) * input + self.dry_wet * phase_shifted
    }

    /// Process mono input, return stereo output
    pub fn process_mono(&mut self, input: f32) -> (f32, f32) {
        self.process(input, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phaser_basic() {
        let mut phaser = Phaser::new(44100.0);
        phaser.set_amount(0.5);
        phaser.set_dry_wet(0.5);
        phaser.set_lfo_freq(1.0);

        for i in 0..10000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let (left, right) = phaser.process_mono(input);
            assert!(left.is_finite() && right.is_finite());
        }
    }

    #[test]
    fn test_phaser_stereo() {
        let mut phaser = Phaser::new(44100.0);
        phaser.set_amount(0.5);
        phaser.set_dry_wet(1.0);
        phaser.set_lfo_freq(2.0);

        // Check stereo spread
        let mut diff_sum = 0.0f32;

        for i in 0..44100 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let (left, right) = phaser.process_mono(input);

            diff_sum += (left - right).abs();
        }

        // There should be stereo difference due to phase offset
        assert!(diff_sum > 1.0, "Phaser should create stereo difference");
    }

    #[test]
    fn test_phaser_feedback() {
        let mut phaser = Phaser::new(44100.0);
        phaser.set_amount(0.5);
        phaser.set_dry_wet(0.5);
        phaser.set_feedback(0.9);

        for i in 0..1000 {
            let input = if i < 10 { 1.0 } else { 0.0 };
            let (left, right) = phaser.process_mono(input);
            assert!(left.is_finite() && right.is_finite());
        }
    }
}
