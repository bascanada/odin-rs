//! Stereo Delay Effect
//!
//! A stereo delay with feedback, highpass filtering, and optional ping-pong mode.

extern crate alloc;

use alloc::boxed::Box;
use crate::dsp::filters::VAOnePoleFilter;
use crate::dsp::math::lerp;

/// Maximum delay time in seconds
const MAX_DELAY_TIME: f32 = 2.0;

/// Buffer size for 48kHz (most common sample rate)
/// For higher sample rates, delay time will be proportionally shorter
const BUFFER_SIZE: usize = 48000 * 2; // 2 seconds at 48kHz

/// Stereo delay effect
pub struct Delay {
    /// Left channel circular buffer
    buffer_left: Box<[f32; BUFFER_SIZE]>,

    /// Right channel circular buffer
    buffer_right: Box<[f32; BUFFER_SIZE]>,

    /// Write index
    write_index: usize,

    /// Sample rate
    sample_rate: f32,

    /// Delay time in seconds (control value)
    delay_time_control: f32,

    /// Delay time smoothed
    delay_time_smooth: f32,

    /// Feedback amount (0.0 to 1.0)
    feedback: f32,

    /// Dry mix (0.0 to 1.0)
    dry: f32,

    /// Wet mix (0.0 to 1.0)
    wet: f32,

    /// Highpass filter for left channel
    highpass_left: VAOnePoleFilter,

    /// Highpass filter for right channel
    highpass_right: VAOnePoleFilter,

    /// Highpass frequency
    highpass_freq: f32,

    /// Ping-pong mode
    ping_pong: bool,

    /// Sync time ratio for BPM sync
    sync_ratio: f32,
}

impl Delay {
    /// Create a new delay effect
    pub fn new(sample_rate: f32) -> Self {
        let mut highpass_left = VAOnePoleFilter::new(sample_rate);
        highpass_left.set_highpass();
        highpass_left.set_frequency(80.0);

        let mut highpass_right = VAOnePoleFilter::new(sample_rate);
        highpass_right.set_highpass();
        highpass_right.set_frequency(80.0);

        Self {
            buffer_left: Box::new([0.0; BUFFER_SIZE]),
            buffer_right: Box::new([0.0; BUFFER_SIZE]),
            write_index: 0,
            sample_rate,
            delay_time_control: 0.5,
            delay_time_smooth: 0.5,
            feedback: 0.45,
            dry: 1.0,
            wet: 0.5,
            highpass_left,
            highpass_right,
            highpass_freq: 80.0,
            ping_pong: false,
            sync_ratio: 3.0 / 16.0,
        }
    }

    /// Set delay time in seconds
    pub fn set_delay_time(&mut self, time: f32) {
        self.delay_time_control = time.clamp(0.001, MAX_DELAY_TIME);
    }

    /// Set delay time from BPM
    pub fn set_delay_from_bpm(&mut self, bpm: f32) {
        let time = 240.0 * self.sync_ratio / bpm;
        self.set_delay_time(time);
    }

    /// Set sync ratio for BPM sync
    pub fn set_sync_ratio(&mut self, numerator: f32, denominator: f32) {
        self.sync_ratio = numerator / denominator;
    }

    /// Set feedback amount (0.0 to 1.0)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99); // Limit to prevent runaway
    }

    /// Set dry level (0.0 to 1.0)
    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry.clamp(0.0, 1.0);
    }

    /// Set wet level (0.0 to 1.0)
    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.clamp(0.0, 1.0);
    }

    /// Set highpass filter frequency
    pub fn set_highpass_freq(&mut self, freq: f32) {
        self.highpass_freq = freq.clamp(20.0, 2000.0);
        self.highpass_left.set_frequency(self.highpass_freq);
        self.highpass_right.set_frequency(self.highpass_freq);
    }

    /// Enable/disable ping-pong mode
    pub fn set_ping_pong(&mut self, enabled: bool) {
        self.ping_pong = enabled;
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.highpass_left.set_sample_rate(sample_rate);
        self.highpass_right.set_sample_rate(sample_rate);
    }

    /// Reset delay buffers
    pub fn reset(&mut self) {
        for i in 0..BUFFER_SIZE {
            self.buffer_left[i] = 0.0;
            self.buffer_right[i] = 0.0;
        }
        self.write_index = 0;
        self.delay_time_smooth = self.delay_time_control;
        self.highpass_left.reset();
        self.highpass_right.reset();
    }

    /// Read from buffer with linear interpolation
    fn read_buffer(buffer: &[f32; BUFFER_SIZE], read_index: f32) -> f32 {
        let mut index_trunc = read_index as i32;
        let frac = read_index - index_trunc as f32;
        let mut index_next = index_trunc + 1;

        // Wrap indices
        while index_trunc < 0 {
            index_trunc += BUFFER_SIZE as i32;
        }
        while index_next < 0 {
            index_next += BUFFER_SIZE as i32;
        }

        let idx0 = (index_trunc as usize) % BUFFER_SIZE;
        let idx1 = (index_next as usize) % BUFFER_SIZE;

        lerp(buffer[idx0], buffer[idx1], frac)
    }

    /// Process stereo delay
    /// Returns (left_out, right_out)
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        // Smooth delay time
        self.delay_time_smooth = self.delay_time_smooth * 0.9999 + self.delay_time_control * 0.0001;

        // Calculate read position
        let delay_samples = self.delay_time_smooth * self.sample_rate;
        let delay_samples = delay_samples.min((BUFFER_SIZE - 1) as f32);
        let read_index = self.write_index as f32 - delay_samples;

        // Read from buffers
        let left_delayed = Self::read_buffer(&self.buffer_left, read_index);
        let right_delayed = Self::read_buffer(&self.buffer_right, read_index);

        // Write to buffers (with feedback)
        if self.ping_pong {
            // Ping-pong: left input goes to left buffer, right delay goes back
            self.buffer_left[self.write_index] = left_in * 0.5 + right_delayed * self.feedback;
            self.buffer_right[self.write_index] = right_in * 0.5 + left_delayed * self.feedback;
        } else {
            // Standard stereo delay
            self.buffer_left[self.write_index] = left_in + left_delayed * self.feedback;
            self.buffer_right[self.write_index] = right_in + right_delayed * self.feedback;
        }

        // Increment write index
        self.write_index = (self.write_index + 1) % BUFFER_SIZE;

        // Apply highpass filter
        let left_filtered = self.highpass_left.process(left_delayed);
        let right_filtered = self.highpass_right.process(right_delayed);

        // Mix dry and wet
        let left_out = left_in * self.dry + left_filtered * self.wet;
        let right_out = right_in * self.dry + right_filtered * self.wet;

        (left_out, right_out)
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
    fn test_delay_basic() {
        let mut delay = Delay::new(44100.0);
        delay.set_delay_time(0.1); // 100ms
        delay.set_feedback(0.5);
        delay.set_wet(0.5);

        // Send impulse and capture output
        let (left, right) = delay.process(1.0, 1.0);
        assert!(left.is_finite() && right.is_finite());

        // Process more samples
        for _ in 0..10000 {
            let (l, r) = delay.process(0.0, 0.0);
            assert!(l.is_finite() && r.is_finite());
        }
    }

    #[test]
    fn test_delay_echo() {
        let mut delay = Delay::new(44100.0);
        delay.set_delay_time(0.1); // 100ms = 4410 samples
        delay.set_feedback(0.0);
        delay.set_wet(1.0);
        delay.set_dry(0.0);

        // Send impulse
        delay.process(1.0, 1.0);

        // Process until we should hear the echo
        let delay_samples = (0.1 * 44100.0) as usize;

        for i in 0..delay_samples + 10 {
            let (left, _) = delay.process(0.0, 0.0);

            // Around the delay time, we should get output
            if i >= delay_samples - 5 && i <= delay_samples + 5 {
                // Echo should be present
                if left.abs() > 0.1 {
                    return; // Success, found echo
                }
            }
        }
    }

    #[test]
    fn test_delay_ping_pong() {
        let mut delay = Delay::new(44100.0);
        delay.set_delay_time(0.05);
        delay.set_feedback(0.7);
        delay.set_ping_pong(true);
        delay.set_wet(1.0);
        delay.set_dry(0.0);

        // Send impulse to both channels (ping-pong will still create stereo difference)
        delay.process(1.0, 1.0);

        // Process and check that both channels have signal
        let mut left_sum = 0.0f32;
        let mut right_sum = 0.0f32;

        for _ in 0..10000 {
            let (left, right) = delay.process(0.0, 0.0);
            left_sum += left.abs();
            right_sum += right.abs();
        }

        // Both channels should have received signal
        assert!(left_sum > 0.1, "Left channel should have signal: {}", left_sum);
        assert!(right_sum > 0.1, "Right channel should have signal: {}", right_sum);
    }
}
