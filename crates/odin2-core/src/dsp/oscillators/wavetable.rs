//! Wavetable Oscillator
//!
//! A wavetable oscillator that reads from pre-computed wavetables.
//! Supports 160 different wavetables with 33 subtables each for anti-aliasing.

use super::Oscillator;
use crate::constants::*;
use crate::dsp::math::lerp;
use crate::wavetables;

/// Wavetable oscillator
///
/// Uses band-limited wavetables for alias-free synthesis.
/// The appropriate subtable is selected based on the current frequency
/// to ensure harmonics don't exceed Nyquist.
pub struct WavetableOscillator {
    /// Current phase (0.0 to WAVETABLE_LENGTH as f32)
    read_index: f32,

    /// Phase increment per sample
    wavetable_inc: f32,

    /// Current frequency in Hz
    frequency: f32,

    /// Sample rate
    sample_rate: f32,

    /// One over sample rate
    one_over_sample_rate: f32,

    /// Current wavetable index (0-159)
    wavetable_index: usize,

    /// Current subtable index (0-32, selected based on frequency)
    subtable_index: usize,

    /// Cached pointer to current subtable data
    current_table: &'static [f32; WAVETABLE_LENGTH],
}

/// Minor third frequency ratios for subtable selection
/// Each subtable covers a range of about a minor third (ratio ~1.19)
const TABLE_FREQUENCIES: [f32; SUBTABLES_PER_WAVETABLE] = [
    25.18, 29.94, 35.61, 42.35, 50.37, 59.91, 71.25, 84.74, 100.77, 119.85, 142.54,
    169.54, 201.60, 239.77, 285.15, 339.16, 403.33, 479.68, 570.47, 678.51, 806.89,
    959.62, 1141.24, 1357.37, 1614.17, 1919.67, 2282.98, 2715.30, 3228.98, 3840.07,
    4566.78, 5431.49, 6458.91,
];

impl WavetableOscillator {
    /// Create a new wavetable oscillator
    pub fn new(sample_rate: f32) -> Self {
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

    /// Set the wavetable by index (0-159)
    pub fn set_wavetable(&mut self, index: usize) {
        self.wavetable_index = index.min(NUMBER_OF_WAVETABLES - 1);
        self.update_subtable();
    }

    /// Set the wavetable by name
    pub fn set_wavetable_by_name(&mut self, name: &str) -> bool {
        for (i, table_name) in wavetables::WAVETABLE_NAMES.iter().enumerate() {
            if *table_name == name {
                self.set_wavetable(i);
                return true;
            }
        }
        false
    }

    /// Get the current wavetable index
    pub fn wavetable_index(&self) -> usize {
        self.wavetable_index
    }

    /// Get the current wavetable name
    pub fn wavetable_name(&self) -> &'static str {
        wavetables::WAVETABLE_NAMES[self.wavetable_index]
    }

    /// Update the subtable based on current frequency
    fn update_subtable(&mut self) {
        // Find the appropriate subtable for the current frequency
        // Higher frequencies need fewer harmonics (higher subtable index)
        let mut new_subtable = 0;

        for (i, &freq) in TABLE_FREQUENCIES.iter().enumerate() {
            if self.frequency < freq {
                break;
            }
            new_subtable = i;
        }

        self.subtable_index = new_subtable;
        self.current_table = wavetables::get_subtable(self.wavetable_index, self.subtable_index);
    }

    /// Linear interpolation read from wavetable
    #[inline]
    fn read_wavetable(&self) -> f32 {
        let index_trunc = self.read_index as usize;
        let fractional = self.read_index - index_trunc as f32;

        let index_next = if index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            index_trunc + 1
        };

        lerp(
            self.current_table[index_trunc],
            self.current_table[index_next],
            fractional,
        )
    }
}

impl Oscillator for WavetableOscillator {
    fn process(&mut self) -> f32 {
        // Read with interpolation
        let output = self.read_wavetable();

        // Advance read index
        self.read_index += self.wavetable_inc;

        // Wrap around
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        output
    }

    fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);

        // Calculate wavetable increment
        // increment = freq * table_length / sample_rate
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;

        // Update subtable selection
        self.update_subtable();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;

        // Recalculate increment
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
    }

    fn reset(&mut self) {
        self.read_index = 0.0;
    }

    fn randomize_phase(&mut self) {
        // Simple pseudo-random
        self.read_index = (self.read_index * 12345.6789) % (WAVETABLE_LENGTH as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavetable_osc_basic() {
        let mut osc = WavetableOscillator::new(44100.0);
        osc.set_wavetable(0); // Sine
        osc.set_frequency(440.0);

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
    fn test_wavetable_switching() {
        let mut osc = WavetableOscillator::new(44100.0);

        // Try different wavetables
        for i in 0..10 {
            osc.set_wavetable(i);
            osc.set_frequency(440.0);

            let sample = osc.process();
            assert!(sample.is_finite(), "Wavetable {} produced invalid output", i);
        }
    }

    #[test]
    fn test_frequency_affects_subtable() {
        let mut osc = WavetableOscillator::new(44100.0);
        osc.set_wavetable(1); // FatSaw

        osc.set_frequency(100.0);
        let low_subtable = osc.subtable_index;

        osc.set_frequency(5000.0);
        let high_subtable = osc.subtable_index;

        // Higher frequency should use higher subtable (fewer harmonics)
        assert!(high_subtable > low_subtable, "Low: {}, High: {}", low_subtable, high_subtable);
    }
}
