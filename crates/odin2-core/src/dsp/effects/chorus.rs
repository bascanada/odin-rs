//! Chorus Effect
//!
//! A stereo chorus effect using two modulated delay lines with LFO modulation.

use crate::dsp::math::lerp;

/// Chorus buffer length (1 second at 44.1kHz)
const CHORUS_BUFFER_LENGTH: usize = 44100;

/// Minimum delay for line 1 (15ms)
const CHORUS_MIN_DISTANCE_1: f32 = 0.015;

/// Minimum delay for line 2 (11ms)
const CHORUS_MIN_DISTANCE_2: f32 = 0.011;

/// Maximum additional modulation range (20ms)
const CHORUS_AMOUNT_RANGE: f32 = 0.02;

/// Chorus effect
pub struct Chorus {
    /// Circular buffer
    buffer: [f32; CHORUS_BUFFER_LENGTH],

    /// Write index
    write_index: usize,

    /// Sample rate
    sample_rate: f32,

    /// LFO position (0.0 to 2.0, for cheap sine approximation)
    lfo_pos: f32,

    /// LFO increment per sample
    lfo_inc: f32,

    /// LFO frequency in Hz
    lfo_freq: f32,

    /// Modulation amount (0.0 to 1.0)
    amount: f32,

    /// Amount control (before smoothing)
    amount_control: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    dry_wet: f32,

    /// Feedback amount
    feedback: f32,

    /// LFO reset position
    lfo_reset_pos: f32,

    /// Sync ratio for BPM
    sync_ratio: f32,

    /// Feedback sample storage
    feedback_sample: f32,
}

impl Chorus {
    /// Create a new chorus effect
    pub fn new(sample_rate: f32) -> Self {
        let lfo_freq = 0.5;
        let lfo_inc = 2.0 * lfo_freq / sample_rate;

        Self {
            buffer: [0.0; CHORUS_BUFFER_LENGTH],
            write_index: 0,
            sample_rate,
            lfo_pos: 0.0,
            lfo_inc,
            lfo_freq,
            amount: 0.16,
            amount_control: 0.4,
            dry_wet: 0.5,
            feedback: 0.0,
            lfo_reset_pos: 0.0,
            sync_ratio: 3.0 / 16.0,
            feedback_sample: 0.0,
        }
    }

    /// Set LFO frequency in Hz
    pub fn set_lfo_freq(&mut self, freq: f32) {
        self.lfo_freq = freq.clamp(0.01, 10.0);
        self.lfo_inc = 2.0 * self.lfo_freq / self.sample_rate;
    }

    /// Set LFO frequency from BPM
    pub fn set_lfo_from_bpm(&mut self, bpm: f32) {
        let freq = bpm / self.sync_ratio / 240.0;
        self.set_lfo_freq(freq);
    }

    /// Set sync ratio for BPM sync
    pub fn set_sync_ratio(&mut self, numerator: f32, denominator: f32) {
        self.sync_ratio = numerator / denominator;
    }

    /// Set modulation amount (0.0 to 1.0)
    pub fn set_amount(&mut self, amount: f32) {
        self.amount_control = amount.clamp(0.0, 1.0);
        self.amount = self.amount_control * self.amount_control; // Square for better feel
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_dry_wet(&mut self, mix: f32) {
        self.dry_wet = mix.clamp(0.0, 1.0);
    }

    /// Set feedback amount (0.0 to 0.9)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.9);
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.lfo_inc = 2.0 * self.lfo_freq / sample_rate;
    }

    /// Reset the chorus
    pub fn reset(&mut self) {
        self.buffer = [0.0; CHORUS_BUFFER_LENGTH];
        self.lfo_pos = self.lfo_reset_pos;
        self.amount = self.amount_control * self.amount_control;
        self.feedback_sample = 0.0;
    }

    /// Calculate cheap sine approximation from LFO position
    /// LFO position is 0.0 to 2.0, output is -1.0 to 1.0
    fn lfo_sine(pos: f32) -> f32 {
        let sign = if pos < 1.0 { 1.0 } else { -1.0 };
        let calc_pos = if pos > 1.0 { pos - 1.0 } else { pos };
        4.0 * calc_pos * (1.0 - calc_pos) * sign
    }

    /// Get both LFO values (90 degrees out of phase)
    fn get_lfo_values(&self) -> (f32, f32) {
        let lfo1 = Self::lfo_sine(self.lfo_pos);

        let mut lfo2_pos = self.lfo_pos + 0.5;
        if lfo2_pos > 2.0 {
            lfo2_pos -= 2.0;
        }
        let lfo2 = Self::lfo_sine(lfo2_pos);

        (lfo1, lfo2)
    }

    /// Increment LFO position
    fn inc_lfo(&mut self) {
        self.lfo_pos += self.lfo_inc;
        while self.lfo_pos >= 2.0 {
            self.lfo_pos -= 2.0;
        }
    }

    /// Read from buffer with linear interpolation
    fn read_buffer(&self, delay_samples: f32) -> f32 {
        let read_pos = self.write_index as f32 - delay_samples;

        let mut index_trunc = read_pos as i32;
        let frac = read_pos - index_trunc as f32;
        let mut index_next = index_trunc + 1;

        while index_trunc < 0 {
            index_trunc += CHORUS_BUFFER_LENGTH as i32;
        }
        while index_next < 0 {
            index_next += CHORUS_BUFFER_LENGTH as i32;
        }

        let idx0 = (index_trunc as usize) % CHORUS_BUFFER_LENGTH;
        let idx1 = (index_next as usize) % CHORUS_BUFFER_LENGTH;

        lerp(self.buffer[idx0], self.buffer[idx1], frac)
    }

    /// Process one sample through the chorus
    /// Returns stereo output (left, right)
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        // Get LFO values
        let (lfo1, lfo2) = self.get_lfo_values();

        // Calculate delay times in samples
        let base_delay_1 = CHORUS_MIN_DISTANCE_1 * self.sample_rate;
        let base_delay_2 = CHORUS_MIN_DISTANCE_2 * self.sample_rate;
        let mod_range = CHORUS_AMOUNT_RANGE * self.amount * self.sample_rate;

        let delay_1 = base_delay_1 + (lfo1 * 0.5 + 0.5) * mod_range;
        let delay_2 = base_delay_2 + (lfo2 * 0.5 + 0.5) * mod_range;

        // Read from delay lines
        let wet1 = self.read_buffer(delay_1);
        let wet2 = self.read_buffer(delay_2);

        // Write to buffer (with feedback)
        self.buffer[self.write_index] = input + self.feedback_sample * self.feedback;

        // Store for next iteration
        self.feedback_sample = (wet1 + wet2) * 0.5;

        // Increment write index
        self.write_index = (self.write_index + 1) % CHORUS_BUFFER_LENGTH;

        // Increment LFO
        self.inc_lfo();

        // Mix dry and wet (stereo output)
        let dry_amount = 1.0 - self.dry_wet;
        let wet_amount = self.dry_wet;

        let left = input * dry_amount + wet1 * wet_amount;
        let right = input * dry_amount + wet2 * wet_amount;

        (left, right)
    }

    /// Process mono to mono (sums stereo output)
    pub fn process_mono(&mut self, input: f32) -> f32 {
        let (left, right) = self.process(input);
        (left + right) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chorus_basic() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_amount(0.5);
        chorus.set_dry_wet(0.5);

        // Process some samples
        for i in 0..10000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let (left, right) = chorus.process(input);
            assert!(left.is_finite() && right.is_finite());
        }
    }

    #[test]
    fn test_chorus_stereo_spread() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_amount(0.5);
        chorus.set_dry_wet(1.0); // Full wet
        chorus.set_lfo_freq(2.0);

        // Process and check that left and right differ
        let mut left_sum = 0.0f32;
        let mut right_sum = 0.0f32;
        let mut diff_sum = 0.0f32;

        for i in 0..44100 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let (left, right) = chorus.process(input);

            left_sum += left.abs();
            right_sum += right.abs();
            diff_sum += (left - right).abs();
        }

        // Both channels should have signal
        assert!(left_sum > 100.0, "Left channel should have signal");
        assert!(right_sum > 100.0, "Right channel should have signal");

        // There should be some stereo difference
        assert!(diff_sum > 1.0, "Chorus should create stereo difference");
    }

    #[test]
    fn test_chorus_dry_wet() {
        let mut chorus = Chorus::new(44100.0);
        chorus.set_amount(0.5);

        // Full dry
        chorus.set_dry_wet(0.0);
        let (dry_l, _dry_r) = chorus.process(1.0);

        chorus.reset();

        // Full wet
        chorus.set_dry_wet(1.0);
        let (wet_l, _) = chorus.process(1.0);

        // Dry should pass through immediately
        assert!((dry_l - 1.0).abs() < 0.01, "Dry signal should pass through");

        // Wet should be different (delayed/modulated)
        // Note: first sample wet might be 0 or close to it due to delay
        assert!(wet_l.is_finite());
    }
}
