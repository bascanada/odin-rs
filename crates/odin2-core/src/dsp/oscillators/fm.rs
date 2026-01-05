//! FM (Frequency Modulation) Oscillator
//!
//! A 2-operator FM oscillator using wavetable-based carrier and modulator.
//! Supports both exponential and linear FM modes.

use super::wavetable::WavetableOscillator;
use super::Oscillator;
use crate::constants::*;
use crate::dsp::math::pitch_shift_multiplier;

/// Exponential FM range in semitones
const EXP_FM_SEMITONES: f32 = 24.0;

/// Linear FM multiplier
const LINEAR_FM_MULTIPLIER: f32 = 15.0;

/// FM mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FmMode {
    /// Linear FM - modulator directly modulates carrier frequency
    #[default]
    Linear,
    /// Exponential FM - modulator modulates carrier pitch in semitones
    Exponential,
}

/// FM Oscillator with carrier and modulator
///
/// Uses two wavetable oscillators in a modulator->carrier configuration.
/// The modulator's output modulates the carrier's frequency (FM) or pitch (exp FM).
pub struct FmOscillator {
    /// Carrier oscillator
    carrier: WavetableOscillator,

    /// Modulator oscillator
    modulator: WavetableOscillator,

    /// Sample rate
    sample_rate: f32,

    /// Base frequency in Hz
    frequency: f32,

    /// Carrier frequency ratio (integer for harmonic ratios)
    carrier_ratio: i32,

    /// Modulator frequency ratio
    modulator_ratio: i32,

    /// FM amount (0.0 to 1.0+)
    fm_amount: f32,

    /// FM mode (linear or exponential)
    fm_mode: FmMode,

    /// Current modulator output (cached for FM calculation)
    mod_output: f32,
}

impl FmOscillator {
    /// Create a new FM oscillator
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier: WavetableOscillator::new(sample_rate),
            modulator: WavetableOscillator::new(sample_rate),
            sample_rate,
            frequency: OSC_FREQ_DEFAULT,
            carrier_ratio: 1,
            modulator_ratio: 1,
            fm_amount: 0.0,
            fm_mode: FmMode::Linear,
            mod_output: 0.0,
        }
    }

    /// Set the FM amount (modulation depth)
    ///
    /// Range: 0.0 = no modulation, 1.0 = full modulation
    /// Can exceed 1.0 for more extreme effects
    pub fn set_fm_amount(&mut self, amount: f32) {
        self.fm_amount = amount.max(0.0);
    }

    /// Get the current FM amount
    pub fn fm_amount(&self) -> f32 {
        self.fm_amount
    }

    /// Set the carrier ratio
    ///
    /// Integer ratios create harmonic relationships.
    /// Carrier freq = base_freq * carrier_ratio
    pub fn set_carrier_ratio(&mut self, ratio: i32) {
        self.carrier_ratio = ratio.max(1);
        self.update_frequencies();
    }

    /// Set the modulator ratio
    ///
    /// Modulator freq = base_freq * modulator_ratio
    pub fn set_modulator_ratio(&mut self, ratio: i32) {
        self.modulator_ratio = ratio.max(1);
        self.update_frequencies();
    }

    /// Set both ratios at once
    pub fn set_ratios(&mut self, carrier: i32, modulator: i32) {
        self.carrier_ratio = carrier.max(1);
        self.modulator_ratio = modulator.max(1);
        self.update_frequencies();
    }

    /// Get the carrier ratio
    pub fn carrier_ratio(&self) -> i32 {
        self.carrier_ratio
    }

    /// Get the modulator ratio
    pub fn modulator_ratio(&self) -> i32 {
        self.modulator_ratio
    }

    /// Set the FM mode
    pub fn set_fm_mode(&mut self, mode: FmMode) {
        if self.fm_mode != mode {
            self.fm_mode = mode;
            // Reset oscillators to avoid discontinuities
            self.carrier.reset();
            self.modulator.reset();
        }
    }

    /// Get the current FM mode
    pub fn fm_mode(&self) -> FmMode {
        self.fm_mode
    }

    /// Set the carrier wavetable
    pub fn set_carrier_wavetable(&mut self, index: usize) {
        self.carrier.set_wavetable(index);
    }

    /// Set the modulator wavetable
    pub fn set_modulator_wavetable(&mut self, index: usize) {
        self.modulator.set_wavetable(index);
    }

    /// Set both wavetables
    pub fn set_wavetables(&mut self, carrier: usize, modulator: usize) {
        self.carrier.set_wavetable(carrier);
        self.modulator.set_wavetable(modulator);
    }

    /// Get carrier wavetable name
    pub fn carrier_wavetable_name(&self) -> &'static str {
        self.carrier.wavetable_name()
    }

    /// Get modulator wavetable name
    pub fn modulator_wavetable_name(&self) -> &'static str {
        self.modulator.wavetable_name()
    }

    /// Update internal frequencies based on ratios
    fn update_frequencies(&mut self) {
        // Modulator frequency = base_freq * modulator_ratio / carrier_ratio
        let mod_freq = self.frequency * (self.modulator_ratio as f32) / (self.carrier_ratio as f32);
        self.modulator.set_frequency(mod_freq);

        // Carrier base frequency (will be modulated during process)
        self.carrier.set_frequency(self.frequency);
    }
}

impl Oscillator for FmOscillator {
    fn process(&mut self) -> f32 {
        // First, process the modulator to get modulation signal
        self.mod_output = self.modulator.process();

        // Apply FM based on mode
        let carrier_freq = match self.fm_mode {
            FmMode::Exponential => {
                // Exponential FM: modulator output shifts pitch in semitones
                let pitch_mod = self.mod_output * self.fm_amount * EXP_FM_SEMITONES;
                self.frequency * pitch_shift_multiplier(pitch_mod)
            }
            FmMode::Linear => {
                // Linear FM: modulator directly adds to carrier frequency
                let freq_mod = self.mod_output * LINEAR_FM_MULTIPLIER * self.frequency * self.fm_amount;
                (self.frequency + freq_mod).max(OSC_FREQ_MIN)
            }
        };

        // Update carrier frequency with modulation
        self.carrier.set_frequency(carrier_freq);

        // Process carrier and return output
        self.carrier.process()
    }

    fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);
        self.update_frequencies();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.carrier.set_sample_rate(sample_rate);
        self.modulator.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
        self.mod_output = 0.0;
    }

    fn randomize_phase(&mut self) {
        self.carrier.randomize_phase();
        self.modulator.randomize_phase();
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_fm_basic() {
        let mut osc = FmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_fm_amount(0.5);
        osc.set_ratios(1, 1);

        // Process some samples
        let mut samples = [0.0f32; 1000];
        for s in &mut samples {
            *s = osc.process();
        }

        // Check that we get valid output
        let max = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0 && max <= 1.1, "Max amplitude: {}", max);
    }

    #[test]
    fn test_fm_ratios() {
        let mut osc = FmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_fm_amount(0.3);

        // Test different harmonic ratios
        for carrier in 1..=4 {
            for modulator in 1..=4 {
                osc.set_ratios(carrier, modulator);

                let sample = osc.process();
                assert!(sample.is_finite(), "Ratio {}:{} produced invalid output", carrier, modulator);
            }
        }
    }

    #[test]
    fn test_fm_modes() {
        let mut osc = FmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_fm_amount(0.5);

        // Linear mode
        osc.set_fm_mode(FmMode::Linear);
        let linear_samples: Vec<f32> = (0..100).map(|_| osc.process()).collect();

        osc.reset();

        // Exponential mode
        osc.set_fm_mode(FmMode::Exponential);
        let exp_samples: Vec<f32> = (0..100).map(|_| osc.process()).collect();

        // They should produce different outputs (at least some difference)
        let diff: f32 = linear_samples.iter()
            .zip(exp_samples.iter())
            .map(|(&a, &b)| (a - b).abs())
            .sum();

        assert!(diff > 0.01, "Linear and exponential modes should produce different outputs");
    }

    #[test]
    fn test_fm_wavetable_selection() {
        let mut osc = FmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_fm_amount(0.5);

        // Set different wavetables
        osc.set_carrier_wavetable(0); // Sine
        osc.set_modulator_wavetable(1); // FatSaw

        assert_eq!(osc.carrier_wavetable_name(), "Sine");
        assert_eq!(osc.modulator_wavetable_name(), "FatSaw");

        let sample = osc.process();
        assert!(sample.is_finite());
    }
}
