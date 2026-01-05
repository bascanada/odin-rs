//! Comb Filter
//!
//! A comb filter creates a series of equally-spaced notches in the frequency spectrum.
//! Used as a building block for flangers and other effects.

extern crate alloc;

use alloc::boxed::Box;
use crate::dsp::filters::dc_blocker::DCBlockingFilter;
use crate::dsp::math::{lerp, pitch_shift_multiplier};

/// Minimum comb frequency (sets maximum delay time)
const COMB_FC_MIN: f32 = 40.0;

/// Buffer length for maximum expected sample rate
const COMB_BUFFER_LENGTH: usize = 192000 / 40 + 100; // ~5000 samples

/// Comb filter
pub struct CombFilter {
    /// Circular buffer
    buffer: Box<[f32; COMB_BUFFER_LENGTH]>,

    /// Write index
    write_index: usize,

    /// Sample rate
    sample_rate: f32,

    /// Delay time in seconds (control value)
    delay_time_control: f32,

    /// Delay time smoothed
    delay_time_smooth: f32,

    /// Feedback amount (-1.0 to 1.0)
    feedback: f32,

    /// Positive or negative comb (1 or -1)
    positive_comb: i8,

    /// DC blocking filter
    dc_blocker: DCBlockingFilter,

    /// Reset smoothing flag
    reset_smoothing: bool,
}

impl CombFilter {
    /// Create a new comb filter
    pub fn new(sample_rate: f32) -> Self {
        let mut dc_blocker = DCBlockingFilter::new();
        dc_blocker.set_sample_rate(sample_rate);

        Self {
            buffer: Box::new([0.0; COMB_BUFFER_LENGTH]),
            write_index: 0,
            sample_rate,
            delay_time_control: 1.0 / 2000.0,
            delay_time_smooth: 1.0 / 2000.0,
            feedback: 0.0,
            positive_comb: 1,
            dc_blocker,
            reset_smoothing: false,
        }
    }

    /// Set the comb frequency in Hz
    pub fn set_frequency(&mut self, freq: f32) {
        let freq = freq.max(COMB_FC_MIN);
        self.delay_time_control = 1.0 / freq;
    }

    /// Set the delay time directly in seconds
    pub fn set_delay_time(&mut self, time: f32) {
        self.delay_time_control = time.max(1.0 / COMB_FC_MIN);
    }

    /// Set feedback amount (-1.0 to 1.0)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-1.0, 1.0);
    }

    /// Alias for set_feedback
    pub fn set_resonance(&mut self, resonance: f32) {
        self.set_feedback(resonance);
    }

    /// Set positive comb (adds) or negative comb (subtracts)
    pub fn set_positive(&mut self, positive: bool) {
        self.positive_comb = if positive { 1 } else { -1 };
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.dc_blocker.set_sample_rate(sample_rate);
    }

    /// Reset the filter
    pub fn reset(&mut self) {
        self.write_index = 0;
        for i in 0..COMB_BUFFER_LENGTH {
            self.buffer[i] = 0.0;
        }
        self.dc_blocker.reset();
        self.delay_time_smooth = self.delay_time_control;
        self.reset_smoothing = true;
    }

    /// Process one sample
    pub fn process(&mut self, input: f32) -> f32 {
        self.process_with_modulation(input, 0.0)
    }

    /// Process one sample with pitch modulation
    pub fn process_with_modulation(&mut self, input: f32, freq_mod_semitones: f32) -> f32 {
        // Handle reset smoothing
        if self.reset_smoothing {
            self.delay_time_smooth = self.delay_time_control;
            self.reset_smoothing = false;
        }

        // Smooth delay time
        self.delay_time_smooth =
            (self.delay_time_smooth - self.delay_time_control) * 0.999 + self.delay_time_control;

        // Apply frequency modulation
        let mut delay_time_modded = self.delay_time_smooth;
        if freq_mod_semitones.abs() > 0.001 {
            delay_time_modded *= pitch_shift_multiplier(-freq_mod_semitones);
        }

        // Clamp delay time
        let max_delay = 1.0 / COMB_FC_MIN;
        delay_time_modded = delay_time_modded.min(max_delay);

        // Calculate read position
        let read_index = self.write_index as f32 - delay_time_modded * self.sample_rate;
        let mut read_index_trunc = libm::floorf(read_index) as i32;
        let mut read_index_next = read_index_trunc + 1;
        let frac = read_index - read_index_trunc as f32;

        // Wrap indices
        while read_index_trunc < 0 {
            read_index_trunc += COMB_BUFFER_LENGTH as i32;
        }
        while read_index_next < 0 {
            read_index_next += COMB_BUFFER_LENGTH as i32;
        }
        while read_index_next >= COMB_BUFFER_LENGTH as i32 {
            read_index_next -= COMB_BUFFER_LENGTH as i32;
        }

        let idx0 = (read_index_trunc as usize) % COMB_BUFFER_LENGTH;
        let idx1 = (read_index_next as usize) % COMB_BUFFER_LENGTH;

        // Linear interpolation for output
        let output = lerp(self.buffer[idx0], self.buffer[idx1], frac);

        // Write to buffer with feedback
        self.buffer[self.write_index] =
            input + output * self.feedback * self.positive_comb as f32;

        // Increment write index
        self.write_index = (self.write_index + 1) % COMB_BUFFER_LENGTH;

        // Mix and apply DC blocking
        let ret = (input + output) * 0.5;
        self.dc_blocker.process(ret)
    }

    /// Get the feedback lower limit (for modulation)
    pub fn feedback_lower_limit(&self) -> f32 {
        0.0
    }

    /// Get the feedback higher limit (for modulation)
    pub fn feedback_higher_limit(&self) -> f32 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comb_basic() {
        let mut filter = CombFilter::new(44100.0);
        filter.set_frequency(440.0);
        filter.set_feedback(0.5);

        // Process some samples
        for i in 0..1000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let output = filter.process(input);
            assert!(output.is_finite(), "Comb filter produced invalid output");
        }
    }

    #[test]
    fn test_comb_feedback() {
        let mut filter = CombFilter::new(44100.0);
        filter.set_frequency(1000.0);
        filter.set_feedback(0.9);

        // Send impulse
        let _ = filter.process(1.0);

        // Process more samples and check for echo
        let mut found_echo = false;
        for _ in 0..500 {
            let output = filter.process(0.0);
            if output.abs() > 0.01 {
                found_echo = true;
            }
        }

        assert!(found_echo, "Comb filter should produce echoes with feedback");
    }

    #[test]
    fn test_comb_negative() {
        let mut filter_pos = CombFilter::new(44100.0);
        filter_pos.set_frequency(1000.0);
        filter_pos.set_feedback(0.5);
        filter_pos.set_positive(true);

        let mut filter_neg = CombFilter::new(44100.0);
        filter_neg.set_frequency(1000.0);
        filter_neg.set_feedback(0.5);
        filter_neg.set_positive(false);

        // Both should produce valid output
        for i in 0..100 {
            let input = if i % 20 < 10 { 0.5 } else { -0.5 };
            let pos_out = filter_pos.process(input);
            let neg_out = filter_neg.process(input);
            assert!(pos_out.is_finite());
            assert!(neg_out.is_finite());
        }
    }
}
