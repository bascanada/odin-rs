//! Ring Modulator Effect
//!
//! A ring modulator multiplies the input signal with an internal oscillator,
//! creating sum and difference frequencies for metallic, bell-like tones.

use crate::constants::WAVETABLE_LENGTH;
use crate::dsp::math::lerp;
use crate::wavetables::get_subtable;

/// Ring Modulator effect
pub struct RingModulator {
    /// Sample rate
    sample_rate: f32,
    /// One over sample rate
    one_over_sample_rate: f32,

    /// Oscillator frequency
    frequency: f32,
    /// Read index into wavetable
    read_index: f32,
    /// Wavetable increment per sample
    wavetable_inc: f32,

    /// Ring modulation amount (0.0 to 1.0)
    amount: f32,

    /// Current wavetable pointer (sine wave, index 0)
    current_table: &'static [f32; WAVETABLE_LENGTH],
}

impl RingModulator {
    /// Create a new ring modulator
    pub fn new(sample_rate: f32) -> Self {
        // Use the first wavetable (sine), first subtable as the modulator
        let current_table = get_subtable(0, 0);

        let frequency = 440.0;
        let wavetable_inc = WAVETABLE_LENGTH as f32 * frequency / sample_rate;

        Self {
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            frequency,
            read_index: 0.0,
            wavetable_inc,
            amount: 1.0,
            current_table,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.update_increment();
    }

    /// Set modulator frequency in Hz
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(0.1, 20000.0);
        self.update_increment();
    }

    /// Set ring modulation amount (0.0 to 1.0)
    /// 0.0 = dry signal only, 1.0 = full ring modulation
    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
    }

    /// Update wavetable increment
    fn update_increment(&mut self) {
        self.wavetable_inc = WAVETABLE_LENGTH as f32 * self.frequency * self.one_over_sample_rate;
    }

    /// Reset the ring modulator
    pub fn reset(&mut self) {
        self.read_index = 0.0;
    }

    /// Read from wavetable with linear interpolation
    fn read_wavetable(&self) -> f32 {
        let read_index_trunc = self.read_index as usize;
        let frac = self.read_index - read_index_trunc as f32;
        let read_index_next = if read_index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            read_index_trunc + 1
        };

        lerp(
            self.current_table[read_index_trunc],
            self.current_table[read_index_next],
            frac,
        )
    }

    /// Process one sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Read modulator signal from wavetable
        let modulator = self.read_wavetable();

        // Increment read index
        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        // Ring modulation: multiply input by modulator
        // Amount controls blend between dry and modulated
        input * (modulator * self.amount + (1.0 - self.amount))
    }

    /// Process stereo input
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Read modulator signal from wavetable (same for both channels)
        let modulator = self.read_wavetable();

        // Increment read index
        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        // Apply to both channels
        let mod_factor = modulator * self.amount + (1.0 - self.amount);
        (left * mod_factor, right * mod_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_mod_basic() {
        let mut ring_mod = RingModulator::new(44100.0);
        ring_mod.set_frequency(440.0);
        ring_mod.set_amount(1.0);

        for i in 0..1000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let output = ring_mod.process(input);
            assert!(output.is_finite(), "Ring mod produced invalid output");
        }
    }

    #[test]
    fn test_ring_mod_amount() {
        let mut ring_mod = RingModulator::new(44100.0);
        ring_mod.set_frequency(440.0);

        // Amount = 0 should pass signal through unchanged
        ring_mod.set_amount(0.0);
        let output = ring_mod.process(0.5);
        assert!(
            (output - 0.5).abs() < 0.01,
            "Amount 0 should pass through: {}",
            output
        );

        ring_mod.reset();

        // Amount = 1 should fully modulate
        ring_mod.set_amount(1.0);
        // Process enough samples to see modulation
        let mut has_variation = false;
        let mut prev = 0.0;
        for _ in 0..1000 {
            let output = ring_mod.process(0.5);
            if (output - prev).abs() > 0.01 {
                has_variation = true;
            }
            prev = output;
        }
        assert!(has_variation, "Full amount should create modulation");
    }

    #[test]
    fn test_ring_mod_frequency() {
        let mut ring_mod = RingModulator::new(44100.0);
        ring_mod.set_amount(1.0);

        // Test different frequencies
        for freq in [100.0, 440.0, 1000.0, 5000.0] {
            ring_mod.reset();
            ring_mod.set_frequency(freq);

            for _ in 0..1000 {
                let output = ring_mod.process(0.5);
                assert!(output.is_finite(), "Ring mod at {}Hz produced invalid output", freq);
            }
        }
    }

    #[test]
    fn test_ring_mod_stereo() {
        let mut ring_mod = RingModulator::new(44100.0);
        ring_mod.set_frequency(440.0);
        ring_mod.set_amount(0.8);

        for i in 0..1000 {
            let left = if i % 100 < 50 { 0.5 } else { -0.5 };
            let right = if i % 80 < 40 { 0.4 } else { -0.4 };
            let (l_out, r_out) = ring_mod.process_stereo(left, right);
            assert!(l_out.is_finite() && r_out.is_finite());
        }
    }
}
