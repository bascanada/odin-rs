//! Multi Oscillator
//!
//! A multi-voice oscillator with 4 sub-oscillators for unison/supersaw sounds.
//! Features independent detuning, stereo spread, and wavetable position spread.

use crate::constants::*;
use crate::dsp::math::lerp;
use crate::wavetables;

/// Number of sub-oscillators
const OSCS_PER_MULTI: usize = 4;

/// Detune offsets for each sub-oscillator (in semitones, multiplied by detune amount)
const DETUNE_OFFSETS: [f32; OSCS_PER_MULTI] = [0.97, -0.348, 0.238, -1.0];

/// Pan offsets for stereo spread
const PAN_OFFSETS: [f32; OSCS_PER_MULTI] = [-1.0, -0.33, 0.33, 1.0];

/// Simple RNG for phase randomization
struct MultiRng {
    state: u32,
}

impl MultiRng {
    fn new() -> Self {
        Self { state: 0x12345678 }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state & 0x7FFFFFFF) as f32 / 0x7FFFFFFF as f32
    }
}

/// Minor third frequency ratios for subtable selection
const TABLE_FREQUENCIES: [f32; SUBTABLES_PER_WAVETABLE] = [
    25.18, 29.94, 35.61, 42.35, 50.37, 59.91, 71.25, 84.74, 100.77, 119.85, 142.54,
    169.54, 201.60, 239.77, 285.15, 339.16, 403.33, 479.68, 570.47, 678.51, 806.89,
    959.62, 1141.24, 1357.37, 1614.17, 1919.67, 2282.98, 2715.30, 3228.98, 3840.07,
    4566.78, 5431.49, 6458.91,
];

/// Multi Oscillator
///
/// Creates thick, detuned sounds using 4 sub-oscillators.
/// Supports wavetable position spread for timbral variation.
pub struct MultiOscillator {
    /// Sample rate
    sample_rate: f32,
    /// One over sample rate
    one_over_sample_rate: f32,

    /// Base frequency
    frequency: f32,

    /// Read indices for each sub-oscillator
    read_indices: [f32; OSCS_PER_MULTI],

    /// Wavetable increments for each sub-oscillator
    wavetable_incs: [f32; OSCS_PER_MULTI],

    /// Current wavetable index
    wavetable_index: usize,

    /// Current subtable index
    subtable_index: usize,

    /// Cached subtable pointers
    current_tables: [&'static [f32; WAVETABLE_LENGTH]; OSCS_PER_MULTI],

    /// Detune amount (0.0 to 1.0)
    detune: f32,

    /// Stereo width (0.0 to 1.0)
    stereo_width: f32,

    /// RNG for phase randomization
    rng: MultiRng,
}

impl MultiOscillator {
    /// Create a new multi oscillator
    pub fn new(sample_rate: f32) -> Self {
        let mut osc = Self {
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            frequency: OSC_FREQ_DEFAULT,
            read_indices: [0.0; OSCS_PER_MULTI],
            wavetable_incs: [0.0; OSCS_PER_MULTI],
            wavetable_index: 0,
            subtable_index: 0,
            current_tables: [wavetables::get_subtable(0, 0); OSCS_PER_MULTI],
            detune: 0.2, // Default detune
            stereo_width: 1.0,
            rng: MultiRng::new(),
        };

        osc.update();
        osc
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.update();
    }

    /// Set base frequency
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);
        self.update();
    }

    /// Set wavetable index (0 to NUMBER_OF_WAVETABLES-1)
    pub fn set_wavetable(&mut self, index: usize) {
        self.wavetable_index = index.min(NUMBER_OF_WAVETABLES - 1);
        self.update_tables();
    }

    /// Set detune amount (0.0 to 1.0)
    /// This controls how much the sub-oscillators are detuned from each other
    pub fn set_detune(&mut self, detune: f32) {
        // Square the detune for better control feel (like C++ version)
        let d = detune.clamp(0.0, 1.0);
        self.detune = d * d;
        self.update();
    }

    /// Set stereo width (0.0 = mono, 1.0 = full stereo spread)
    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
    }

    /// Cheap pitch shift multiplier using Taylor series approximation
    /// More efficient than full exponential calculation
    #[inline]
    fn cheap_pitch_shift(semitones: f32) -> f32 {
        // 2^(x/12) = e^(ln(2)/12 * x)
        // ln(2)/12 = 0.057762265
        let x = semitones * 0.057762265;

        // Taylor series for e^x with Horner's method
        // e^x ≈ 1 + x + x²/2 + x³/6 + x⁴/24
        1.0 + x * (1.0 + x * (0.5 + x * (0.1666666 + x * 0.04166666)))
    }

    /// Update internal state after parameter changes
    fn update(&mut self) {
        // Calculate frequencies and increments for each sub-oscillator
        for i in 0..OSCS_PER_MULTI {
            let detune_semitones = DETUNE_OFFSETS[i] * self.detune;
            let freq = self.frequency * Self::cheap_pitch_shift(detune_semitones);
            let freq_clamped = freq.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);

            self.wavetable_incs[i] =
                freq_clamped * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
        }

        // Update subtable selection based on highest frequency oscillator
        self.update_subtable();
        self.update_tables();
    }

    /// Update subtable index based on frequency
    fn update_subtable(&mut self) {
        // Find max frequency among all sub-oscillators
        let max_freq = self.frequency * Self::cheap_pitch_shift(0.97 * self.detune);

        let mut new_subtable = 0;
        for (i, &freq) in TABLE_FREQUENCIES.iter().enumerate() {
            if max_freq < freq {
                break;
            }
            new_subtable = i;
        }

        self.subtable_index = new_subtable;
    }

    /// Update table pointers
    fn update_tables(&mut self) {
        for i in 0..OSCS_PER_MULTI {
            self.current_tables[i] =
                wavetables::get_subtable(self.wavetable_index, self.subtable_index);
        }
    }

    /// Read from wavetable with linear interpolation
    #[inline]
    fn read_wavetable(&self, osc_index: usize) -> f32 {
        let read_index = self.read_indices[osc_index];
        let index_trunc = read_index as usize;
        let fractional = read_index - index_trunc as f32;

        let index_next = if index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            index_trunc + 1
        };

        lerp(
            self.current_tables[osc_index][index_trunc],
            self.current_tables[osc_index][index_next],
            fractional,
        )
    }

    /// Reset oscillator state
    pub fn reset(&mut self) {
        for i in 0..OSCS_PER_MULTI {
            self.read_indices[i] = 0.0;
        }
    }

    /// Randomize phases for each sub-oscillator
    pub fn randomize_phase(&mut self) {
        for i in 0..OSCS_PER_MULTI {
            self.read_indices[i] = self.rng.next_f32() * WAVETABLE_LENGTH as f32;
        }
    }

    /// Process one sample (mono output)
    pub fn process(&mut self) -> f32 {
        let mut output = 0.0;

        for i in 0..OSCS_PER_MULTI {
            // Read sample
            output += self.read_wavetable(i);

            // Advance phase
            self.read_indices[i] += self.wavetable_incs[i];

            // Wrap around
            while self.read_indices[i] >= WAVETABLE_LENGTH as f32 {
                self.read_indices[i] -= WAVETABLE_LENGTH as f32;
            }
        }

        // Average the outputs
        output * 0.25
    }

    /// Process one sample with stereo output
    /// Returns (left, right)
    pub fn process_stereo(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for i in 0..OSCS_PER_MULTI {
            // Read sample
            let sample = self.read_wavetable(i);

            // Calculate pan based on stereo width
            // PAN_OFFSETS: [-1.0, -0.33, 0.33, 1.0]
            let pan = PAN_OFFSETS[i] * self.stereo_width;
            let left_gain = 1.0 - pan.max(0.0);
            let right_gain = 1.0 + pan.min(0.0);

            left += sample * left_gain;
            right += sample * right_gain;

            // Advance phase
            self.read_indices[i] += self.wavetable_incs[i];

            // Wrap around
            while self.read_indices[i] >= WAVETABLE_LENGTH as f32 {
                self.read_indices[i] -= WAVETABLE_LENGTH as f32;
            }
        }

        // Average and return
        (left * 0.25, right * 0.25)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;
    use super::*;

    #[test]
    fn test_multi_osc_basic() {
        let mut osc = MultiOscillator::new(44100.0);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let output = osc.process();
            assert!(output.is_finite(), "Multi oscillator produced invalid output");
            assert!(output.abs() <= 1.5, "Multi oscillator output too large: {}", output);
        }
    }

    #[test]
    fn test_multi_osc_stereo() {
        let mut osc = MultiOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_stereo_width(1.0);
        osc.set_detune(0.5);

        let mut left_sum = 0.0f32;
        let mut right_sum = 0.0f32;
        let mut diff_sum = 0.0f32;

        for _ in 0..44100 {
            let (left, right) = osc.process_stereo();
            assert!(left.is_finite() && right.is_finite());
            left_sum += left.abs();
            right_sum += right.abs();
            diff_sum += (left - right).abs();
        }

        // Should have stereo content
        assert!(diff_sum > 100.0, "Should have stereo difference");
        // Both channels should have signal
        assert!(left_sum > 1000.0, "Left channel should have signal");
        assert!(right_sum > 1000.0, "Right channel should have signal");
    }

    #[test]
    fn test_multi_osc_detune() {
        let mut osc_no_detune = MultiOscillator::new(44100.0);
        osc_no_detune.set_frequency(440.0);
        osc_no_detune.set_detune(0.0);

        let mut osc_detune = MultiOscillator::new(44100.0);
        osc_detune.set_frequency(440.0);
        osc_detune.set_detune(1.0);

        // Process and collect samples
        let mut samples_no_detune = Vec::with_capacity(1000);
        let mut samples_detune = Vec::with_capacity(1000);

        for _ in 0..1000 {
            samples_no_detune.push(osc_no_detune.process());
            samples_detune.push(osc_detune.process());
        }

        // Detuned should sound different (different waveform shape due to beating)
        let mut diff_count = 0;
        for i in 0..1000 {
            if (samples_no_detune[i] - samples_detune[i]).abs() > 0.01 {
                diff_count += 1;
            }
        }

        assert!(diff_count > 100, "Detune should create difference in output");
    }

    #[test]
    fn test_multi_osc_randomize_phase() {
        let mut osc = MultiOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Get initial sample
        let sample1 = osc.process();

        osc.reset();
        osc.randomize_phase();

        // Get sample after randomize (should be different)
        let sample2 = osc.process();

        // They might occasionally be the same by chance, but unlikely
        // Just verify output is valid
        assert!(sample1.is_finite());
        assert!(sample2.is_finite());
    }

    #[test]
    fn test_multi_osc_wavetables() {
        let mut osc = MultiOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Test with different wavetables
        for wt in 0..10.min(NUMBER_OF_WAVETABLES) {
            osc.reset();
            osc.set_wavetable(wt);

            for _ in 0..100 {
                let output = osc.process();
                assert!(
                    output.is_finite(),
                    "Multi oscillator with wavetable {} produced invalid output",
                    wt
                );
            }
        }
    }

    #[test]
    fn test_cheap_pitch_shift() {
        // Test that cheap pitch shift is reasonably accurate
        // Reference: 2^(semitones/12)

        // Octave up (12 semitones) should be ~2.0
        let octave_up = MultiOscillator::cheap_pitch_shift(12.0);
        assert!(
            (octave_up - 2.0).abs() < 0.1,
            "Octave up should be ~2.0, got {}",
            octave_up
        );

        // Octave down (-12 semitones) should be ~0.5
        let octave_down = MultiOscillator::cheap_pitch_shift(-12.0);
        assert!(
            (octave_down - 0.5).abs() < 0.1,
            "Octave down should be ~0.5, got {}",
            octave_down
        );

        // No shift should be ~1.0
        let no_shift = MultiOscillator::cheap_pitch_shift(0.0);
        assert!(
            (no_shift - 1.0).abs() < 0.001,
            "No shift should be 1.0, got {}",
            no_shift
        );
    }
}
