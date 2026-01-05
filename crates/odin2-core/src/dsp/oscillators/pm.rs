//! PM (Phase Modulation) Oscillator
//!
//! A 2-operator PM oscillator using wavetable-based carrier and modulator.
//! Phase modulation creates similar timbres to FM but with different harmonic relationships.

use super::Oscillator;
use crate::constants::*;
use crate::dsp::math::lerp;
use crate::wavetables;

/// Minor third frequency ratios for subtable selection
/// Same as WavetableOscillator but needed here for PM carrier
const TABLE_FREQUENCIES: [f32; SUBTABLES_PER_WAVETABLE] = [
    25.18, 29.94, 35.61, 42.35, 50.37, 59.91, 71.25, 84.74, 100.77, 119.85, 142.54,
    169.54, 201.60, 239.77, 285.15, 339.16, 403.33, 479.68, 570.47, 678.51, 806.89,
    959.62, 1141.24, 1357.37, 1614.17, 1919.67, 2282.98, 2715.30, 3228.98, 3840.07,
    4566.78, 5431.49, 6458.91,
];

/// PM Carrier oscillator with phase modulation support
///
/// This is a specialized wavetable oscillator that supports phase modulation
/// with proper anti-aliasing based on instantaneous frequency.
struct PmCarrierOscillator {
    /// Current read index (0.0 to WAVETABLE_LENGTH as f32)
    read_index: f32,

    /// Wavetable increment per sample
    wavetable_inc: f32,

    /// Current frequency in Hz
    frequency: f32,

    /// Sample rate
    sample_rate: f32,

    /// One over sample rate
    one_over_sample_rate: f32,

    /// Current wavetable index (0-159)
    wavetable_index: usize,

    /// Current subtable index (0-32)
    subtable_index: usize,

    /// Cached pointer to current subtable data
    current_table: &'static [f32; WAVETABLE_LENGTH],

    /// Current phase modulation value
    phase_mod: f32,

    /// Previous phase modulation value (for velocity calculation)
    last_phase_mod: f32,

    /// Phase velocity (derivative of phase modulation)
    phase_velocity: f32,
}

impl PmCarrierOscillator {
    fn new(sample_rate: f32) -> Self {
        Self {
            read_index: 0.0,
            wavetable_inc: 0.0,
            frequency: OSC_FREQ_DEFAULT,
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            wavetable_index: 0,
            subtable_index: 0,
            current_table: wavetables::get_subtable(0, 0),
            phase_mod: 0.0,
            last_phase_mod: 0.0,
            phase_velocity: 0.0,
        }
    }

    /// Set the phase modulation amount
    ///
    /// This calculates the phase velocity for proper anti-aliasing.
    fn set_phase_mod(&mut self, phase: f32) {
        self.phase_velocity = phase - self.last_phase_mod;
        self.last_phase_mod = self.phase_mod;
        self.phase_mod = phase;
    }

    /// Set the wavetable by index
    fn set_wavetable(&mut self, index: usize) {
        self.wavetable_index = index.min(NUMBER_OF_WAVETABLES - 1);
        self.update_subtable();
    }

    /// Get the wavetable name
    fn wavetable_name(&self) -> &'static str {
        wavetables::WAVETABLE_NAMES[self.wavetable_index]
    }

    /// Get subtable index based on instantaneous frequency
    ///
    /// For PM, we need to consider the phase velocity to calculate
    /// the instantaneous frequency for proper anti-aliasing.
    fn get_table_index(&self) -> usize {
        // Instantaneous frequency: w_i = w + d(phase)/dt
        // where d(phase)/dt = phase_velocity * sample_rate
        let abs_freq = libm::fabsf(self.frequency + self.phase_velocity * self.sample_rate);

        for (i, &freq) in TABLE_FREQUENCIES.iter().enumerate() {
            if abs_freq < freq {
                return i;
            }
        }

        SUBTABLES_PER_WAVETABLE - 1
    }

    /// Update the subtable based on current frequency
    fn update_subtable(&mut self) {
        self.subtable_index = self.get_table_index();
        self.current_table = wavetables::get_subtable(self.wavetable_index, self.subtable_index);
    }

    /// Update internal state
    fn update(&mut self) {
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
        self.subtable_index = self.get_table_index();
        self.current_table = wavetables::get_subtable(self.wavetable_index, self.subtable_index);
    }

    /// Process one sample with phase modulation
    fn process(&mut self) -> f32 {
        // Calculate read index with phase modulation
        let pm_offset = self.phase_mod * (WAVETABLE_LENGTH as f32);
        let modulated_index = self.read_index + pm_offset;

        // Get integer and fractional parts
        let mut index_trunc = modulated_index as i32;
        let fractional = modulated_index - index_trunc as f32;
        let mut index_next = index_trunc + 1;

        // Wrap to valid range
        let table_len = WAVETABLE_LENGTH as i32;
        while index_trunc < 0 {
            index_trunc += table_len;
        }
        while index_trunc >= table_len {
            index_trunc -= table_len;
        }
        while index_next < 0 {
            index_next += table_len;
        }
        while index_next >= table_len {
            index_next -= table_len;
        }

        // Linear interpolation
        let output = lerp(
            self.current_table[index_trunc as usize],
            self.current_table[index_next as usize],
            fractional,
        );

        // Advance read index (without phase modulation - that's added during read)
        self.read_index += self.wavetable_inc;

        // Wrap around
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }
        while self.read_index < 0.0 {
            self.read_index += WAVETABLE_LENGTH as f32;
        }

        output
    }

    fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);
        self.update();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.update();
    }

    fn reset(&mut self) {
        self.read_index = 0.0;
        self.phase_mod = 0.0;
        self.last_phase_mod = 0.0;
        self.phase_velocity = 0.0;
    }

    fn randomize_phase(&mut self) {
        self.read_index = (self.read_index * 12345.6789) % (WAVETABLE_LENGTH as f32);
    }
}

/// Simple modulator oscillator (standard wavetable)
struct PmModulatorOscillator {
    read_index: f32,
    wavetable_inc: f32,
    frequency: f32,
    sample_rate: f32,
    one_over_sample_rate: f32,
    wavetable_index: usize,
    subtable_index: usize,
    current_table: &'static [f32; WAVETABLE_LENGTH],
}

impl PmModulatorOscillator {
    fn new(sample_rate: f32) -> Self {
        Self {
            read_index: 0.0,
            wavetable_inc: 0.0,
            frequency: OSC_FREQ_DEFAULT,
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            wavetable_index: 0,
            subtable_index: 0,
            current_table: wavetables::get_subtable(0, 0),
        }
    }

    fn set_wavetable(&mut self, index: usize) {
        self.wavetable_index = index.min(NUMBER_OF_WAVETABLES - 1);
        self.update_subtable();
    }

    fn wavetable_name(&self) -> &'static str {
        wavetables::WAVETABLE_NAMES[self.wavetable_index]
    }

    fn update_subtable(&mut self) {
        let mut subtable = 0;
        for (i, &freq) in TABLE_FREQUENCIES.iter().enumerate() {
            if self.frequency < freq {
                break;
            }
            subtable = i;
        }
        self.subtable_index = subtable;
        self.current_table = wavetables::get_subtable(self.wavetable_index, self.subtable_index);
    }

    fn process(&mut self) -> f32 {
        let index_trunc = self.read_index as usize;
        let fractional = self.read_index - index_trunc as f32;
        let index_next = if index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            index_trunc + 1
        };

        let output = lerp(
            self.current_table[index_trunc],
            self.current_table[index_next],
            fractional,
        );

        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        output
    }

    fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
        self.update_subtable();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
    }

    fn reset(&mut self) {
        self.read_index = 0.0;
    }

    fn randomize_phase(&mut self) {
        self.read_index = (self.read_index * 12345.6789) % (WAVETABLE_LENGTH as f32);
    }
}

/// PM Oscillator with carrier and modulator
///
/// Uses two wavetable oscillators in a modulator->carrier configuration.
/// The modulator's output modulates the carrier's phase.
pub struct PmOscillator {
    /// Carrier oscillator (with PM support)
    carrier: PmCarrierOscillator,

    /// Modulator oscillator
    modulator: PmModulatorOscillator,

    /// Sample rate
    sample_rate: f32,

    /// Base frequency in Hz
    frequency: f32,

    /// Carrier frequency ratio
    carrier_ratio: i32,

    /// Modulator frequency ratio
    modulator_ratio: i32,

    /// PM amount (0.0 to 1.0+)
    pm_amount: f32,
}

impl PmOscillator {
    /// Create a new PM oscillator
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier: PmCarrierOscillator::new(sample_rate),
            modulator: PmModulatorOscillator::new(sample_rate),
            sample_rate,
            frequency: OSC_FREQ_DEFAULT,
            carrier_ratio: 1,
            modulator_ratio: 1,
            pm_amount: 0.0,
        }
    }

    /// Set the PM amount (modulation depth)
    ///
    /// Range: 0.0 = no modulation, 1.0 = full cycle modulation
    /// Can be negative for inverted modulation
    pub fn set_pm_amount(&mut self, amount: f32) {
        self.pm_amount = amount;
    }

    /// Get the current PM amount
    pub fn pm_amount(&self) -> f32 {
        self.pm_amount
    }

    /// Set the carrier ratio
    pub fn set_carrier_ratio(&mut self, ratio: i32) {
        self.carrier_ratio = ratio.max(1);
        self.update_frequencies();
    }

    /// Set the modulator ratio
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

        // Carrier frequency
        self.carrier.set_frequency(self.frequency);
    }
}

impl Oscillator for PmOscillator {
    fn process(&mut self) -> f32 {
        // First, process the modulator to get modulation signal
        let mod_output = self.modulator.process();

        // Apply phase modulation to carrier
        // PM amount controls how much the modulator affects the carrier's phase
        self.carrier.set_phase_mod(self.pm_amount * mod_output);

        // Update carrier for anti-aliasing (considers phase velocity)
        self.carrier.update();

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
    fn test_pm_basic() {
        let mut osc = PmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_pm_amount(0.5);
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
    fn test_pm_ratios() {
        let mut osc = PmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_pm_amount(0.3);

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
    fn test_pm_negative_amount() {
        let mut osc = PmOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Positive modulation
        osc.set_pm_amount(0.5);
        let pos_samples: Vec<f32> = (0..100).map(|_| osc.process()).collect();

        osc.reset();

        // Negative modulation (inverted)
        osc.set_pm_amount(-0.5);
        let neg_samples: Vec<f32> = (0..100).map(|_| osc.process()).collect();

        // They should produce different outputs
        let diff: f32 = pos_samples.iter()
            .zip(neg_samples.iter())
            .map(|(&a, &b)| (a - b).abs())
            .sum();

        assert!(diff > 0.01, "Positive and negative PM should produce different outputs");
    }

    #[test]
    fn test_pm_wavetable_selection() {
        let mut osc = PmOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_pm_amount(0.5);

        // Set different wavetables
        osc.set_carrier_wavetable(0); // Sine
        osc.set_modulator_wavetable(1); // FatSaw

        assert_eq!(osc.carrier_wavetable_name(), "Sine");
        assert_eq!(osc.modulator_wavetable_name(), "FatSaw");

        let sample = osc.process();
        assert!(sample.is_finite());
    }

    #[test]
    fn test_pm_vs_no_modulation() {
        let mut osc = PmOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // No modulation
        osc.set_pm_amount(0.0);
        let no_mod: Vec<f32> = (0..100).map(|_| osc.process()).collect();

        osc.reset();

        // With modulation
        osc.set_pm_amount(0.5);
        let with_mod: Vec<f32> = (0..100).map(|_| osc.process()).collect();

        // They should produce different outputs
        let diff: f32 = no_mod.iter()
            .zip(with_mod.iter())
            .map(|(&a, &b)| (a - b).abs())
            .sum();

        assert!(diff > 0.1, "Modulated output should differ from unmodulated");
    }
}
