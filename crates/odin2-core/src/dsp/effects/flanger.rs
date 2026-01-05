//! Flanger Effect
//!
//! A flanger effect using a modulated comb filter.
//! Creates the characteristic jet-like swooshing sound.

use crate::dsp::filters::CombFilter;

/// Maximum LFO modulation range in seconds
const FLANGER_LFO_MAX_RANGE: f32 = 0.0095;

/// Flanger effect
pub struct Flanger {
    /// Underlying comb filter
    comb: CombFilter,

    /// Sample rate
    sample_rate: f32,

    /// LFO position (0.0 to 1.0)
    lfo_pos: f32,
    /// LFO increment per sample
    lfo_inc: f32,
    /// LFO frequency in Hz
    lfo_freq: f32,
    /// LFO sign (+1 or -1)
    lfo_sign: i8,
    /// LFO reset position
    lfo_reset_pos: f32,

    /// Base delay time in seconds
    base_time: f32,
    /// LFO modulation amount (0.0 to 1.0)
    amount: f32,
    /// Dry/wet mix (0.0 to 1.0)
    dry_wet: f32,

    /// Sync time ratio
    sync_ratio: f32,
}

impl Flanger {
    /// Create a new flanger effect
    pub fn new(sample_rate: f32) -> Self {
        let mut comb = CombFilter::new(sample_rate);
        comb.set_feedback(0.6);

        let lfo_freq = 0.2;
        let lfo_inc = lfo_freq / sample_rate * 2.0;

        Self {
            comb,
            sample_rate,
            lfo_pos: 0.0,
            lfo_inc,
            lfo_freq,
            lfo_sign: 1,
            lfo_reset_pos: 0.0,
            base_time: 0.0105,
            amount: 0.3,
            dry_wet: 1.0,
            sync_ratio: 3.0 / 16.0,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.comb.set_sample_rate(sample_rate);
        self.set_lfo_freq(self.lfo_freq);
    }

    /// Set LFO frequency in Hz
    pub fn set_lfo_freq(&mut self, freq: f32) {
        self.lfo_freq = freq.clamp(0.01, 20.0);
        self.lfo_inc = self.lfo_freq / self.sample_rate * 2.0;
    }

    /// Set LFO frequency from BPM
    pub fn set_lfo_from_bpm(&mut self, bpm: f32) {
        let freq = bpm / self.sync_ratio / 240.0;
        self.set_lfo_freq(freq);
    }

    /// Set sync time ratio
    pub fn set_sync_ratio(&mut self, numerator: f32, denominator: f32) {
        self.sync_ratio = numerator / denominator;
    }

    /// Set base delay time in seconds
    pub fn set_base_time(&mut self, time: f32) {
        self.base_time = time.clamp(0.001, 0.05);
    }

    /// Set LFO modulation amount (0.0 to 1.0)
    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
    }

    /// Set dry/wet mix (0.0 to 1.0)
    pub fn set_dry_wet(&mut self, mix: f32) {
        self.dry_wet = mix.clamp(0.0, 1.0);
    }

    /// Set feedback amount (-1.0 to 1.0)
    pub fn set_feedback(&mut self, feedback: f32) {
        let feedback = feedback.clamp(-0.98, 0.98);
        self.comb.set_feedback(feedback);
    }

    /// Set LFO reset position
    pub fn set_lfo_reset_pos(&mut self, pos: f32) {
        self.lfo_reset_pos = pos.clamp(0.0, 1.0);
        self.reset_lfo();
    }

    /// Reset the flanger
    pub fn reset(&mut self) {
        self.comb.reset();
        self.reset_lfo();
    }

    /// Reset just the LFO
    pub fn reset_lfo(&mut self) {
        self.lfo_pos = self.lfo_reset_pos;
        self.lfo_sign = 1;
    }

    /// Cheap sine approximation using parabola
    fn lfo_sine(&self) -> f32 {
        4.0 * self.lfo_pos * (1.0 - self.lfo_pos) * self.lfo_sign as f32
    }

    /// Increment LFO position
    fn increment_lfo(&mut self) {
        self.lfo_pos += self.lfo_inc;
        while self.lfo_pos > 1.0 {
            self.lfo_pos -= 1.0;
            self.lfo_sign *= -1;
        }
    }

    /// Process one sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Increment LFO
        self.increment_lfo();
        let lfo = self.lfo_sine();

        // Calculate modulated delay time
        let delay_time = self.base_time + lfo * self.amount * FLANGER_LFO_MAX_RANGE;
        self.comb.set_delay_time(delay_time);

        // Process through comb filter
        let wet = self.comb.process(input);

        // Mix dry and wet
        self.dry_wet * wet + (1.0 - self.dry_wet) * input
    }

    /// Process stereo input (both channels with same settings)
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Note: This is a simple stereo flanger where both channels share the LFO
        // For true stereo, you'd want two separate flangers with phase offset
        let left_out = self.process(left);

        // Process right with same LFO state (mono flanger behavior)
        let delay_time =
            self.base_time + self.lfo_sine() * self.amount * FLANGER_LFO_MAX_RANGE;
        self.comb.set_delay_time(delay_time);
        let wet = self.comb.process(right);
        let right_out = self.dry_wet * wet + (1.0 - self.dry_wet) * right;

        (left_out, right_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flanger_basic() {
        let mut flanger = Flanger::new(44100.0);
        flanger.set_amount(0.5);
        flanger.set_dry_wet(0.5);
        flanger.set_lfo_freq(1.0);

        for i in 0..10000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let output = flanger.process(input);
            assert!(output.is_finite(), "Flanger produced invalid output");
        }
    }

    #[test]
    fn test_flanger_feedback() {
        let mut flanger = Flanger::new(44100.0);
        flanger.set_amount(0.5);
        flanger.set_dry_wet(1.0);
        flanger.set_feedback(0.9);

        // Send impulse and check output
        let _ = flanger.process(1.0);

        let mut has_output = false;
        for _ in 0..1000 {
            let output = flanger.process(0.0);
            if output.abs() > 0.001 {
                has_output = true;
            }
            assert!(output.is_finite());
        }

        assert!(has_output, "Flanger with feedback should produce output");
    }

    #[test]
    fn test_flanger_negative_feedback() {
        let mut flanger = Flanger::new(44100.0);
        flanger.set_amount(0.5);
        flanger.set_dry_wet(1.0);
        flanger.set_feedback(-0.8);

        for i in 0..1000 {
            let input = if i % 100 < 50 { 0.5 } else { -0.5 };
            let output = flanger.process(input);
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_flanger_lfo_sweep() {
        let mut flanger = Flanger::new(44100.0);
        flanger.set_amount(1.0);
        flanger.set_dry_wet(1.0);
        flanger.set_lfo_freq(5.0); // Fast sweep

        // Generate a sine wave input
        let mut energy_variation = 0.0f32;
        let mut prev_output = 0.0f32;

        for i in 0..44100 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let output = flanger.process(input);

            // Track variation in output
            energy_variation += (output - prev_output).abs();
            prev_output = output;
        }

        // Flanger should create variation (modulation)
        assert!(
            energy_variation > 100.0,
            "Flanger should create modulation: {}",
            energy_variation
        );
    }
}
