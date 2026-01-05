//! LFO (Low Frequency Oscillator)
//!
//! A wavetable-based LFO with optional sample-and-hold mode.
//! Used for modulation purposes, always uses the lowest subtable (no anti-aliasing needed).

use super::Oscillator;
use crate::constants::*;
use crate::dsp::math::lerp;
use crate::wavetables;

/// LFO waveform types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LfoWaveform {
    #[default]
    Sine = 0,
    Saw = 1,
    Triangle = 2,
    Square = 3,
    SampleAndHold = 4,
}

/// Simple PRNG for noise/S&H (xorshift32)
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Returns bipolar random value in range [-1.0, 1.0]
    fn next_bipolar(&mut self) -> f32 {
        let n = self.next();
        (n as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Low Frequency Oscillator
///
/// A wavetable-based LFO that can use various waveforms for modulation.
/// Since it's an LFO, we always use the lowest subtable (no anti-aliasing needed at low frequencies).
pub struct Lfo {
    /// Current read index
    read_index: f32,

    /// Wavetable increment per sample
    wavetable_inc: f32,

    /// Current frequency in Hz
    frequency: f32,

    /// Sample rate
    sample_rate: f32,

    /// One over sample rate
    one_over_sample_rate: f32,

    /// Current wavetable index
    wavetable_index: usize,

    /// Cached pointer to current subtable (always subtable 0 for LFO)
    current_table: &'static [f32; WAVETABLE_LENGTH],

    /// LFO waveform
    waveform: LfoWaveform,

    /// Sample and hold value
    sh_value: f32,

    /// Last phase for S&H trigger detection
    last_phase: f32,

    /// Random number generator for S&H
    rng: SimpleRng,

    /// BPM sync ratio (numerator / denominator)
    sync_ratio: f32,
}

impl Lfo {
    /// Create a new LFO
    pub fn new(sample_rate: f32) -> Self {
        let mut rng = SimpleRng::new(12345);
        let sh_value = rng.next_bipolar();

        Self {
            read_index: 0.0,
            wavetable_inc: 0.0,
            frequency: 1.0, // Default 1 Hz
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            wavetable_index: 0, // Sine
            current_table: wavetables::get_subtable(0, 0), // Always use lowest subtable
            waveform: LfoWaveform::Sine,
            sh_value,
            last_phase: 0.0,
            rng,
            sync_ratio: 3.0 / 16.0, // Default sync
        }
    }

    /// Set LFO frequency in Hz
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.max(0.001).min(100.0); // LFO range: 0.001 Hz to 100 Hz
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
    }

    /// Set LFO frequency from BPM
    pub fn set_freq_from_bpm(&mut self, bpm: f32) {
        let freq = bpm / self.sync_ratio / 240.0;
        self.set_frequency(freq);
    }

    /// Set sync time ratio
    pub fn set_sync_ratio(&mut self, numerator: f32, denominator: f32) {
        self.sync_ratio = numerator / denominator;
    }

    /// Set the waveform
    pub fn set_waveform(&mut self, waveform: LfoWaveform) {
        self.waveform = waveform;

        // Update wavetable based on waveform
        let wt_index = match waveform {
            LfoWaveform::Sine => 0,
            LfoWaveform::Saw => 1,
            LfoWaveform::Triangle => 2,
            LfoWaveform::Square => 3,
            LfoWaveform::SampleAndHold => 0, // Not used for S&H
        };

        self.wavetable_index = wt_index;
        self.current_table = wavetables::get_subtable(wt_index, 0);
    }

    /// Get the current waveform
    pub fn waveform(&self) -> LfoWaveform {
        self.waveform
    }

    /// Read from wavetable with linear interpolation
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

    /// Process sample and hold
    fn process_sample_and_hold(&mut self) -> f32 {
        // Detect phase wrap (new cycle)
        let current_phase = self.read_index / (WAVETABLE_LENGTH as f32);
        if current_phase < self.last_phase {
            // Phase wrapped, get new random value
            self.sh_value = self.rng.next_bipolar();
        }
        self.last_phase = current_phase;

        self.sh_value
    }
}

impl Oscillator for Lfo {
    fn process(&mut self) -> f32 {
        let output = match self.waveform {
            LfoWaveform::SampleAndHold => self.process_sample_and_hold(),
            _ => self.read_wavetable(),
        };

        // Advance phase
        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        output
    }

    fn set_frequency(&mut self, freq: f32) {
        Lfo::set_frequency(self, freq);
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.wavetable_inc = self.frequency * (WAVETABLE_LENGTH as f32) * self.one_over_sample_rate;
    }

    fn reset(&mut self) {
        self.read_index = 0.0;
        self.last_phase = 0.0;
        self.sh_value = self.rng.next_bipolar();
    }

    fn randomize_phase(&mut self) {
        self.read_index = (self.rng.next() as f32 / u32::MAX as f32) * (WAVETABLE_LENGTH as f32);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_lfo_basic() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_frequency(1.0);

        // Process one second of samples
        let mut samples = [0.0f32; 44100];
        for s in &mut samples {
            *s = lfo.process();
        }

        // Check that we get valid output in range [-1, 1]
        let max = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0 && max <= 1.1, "Max amplitude: {}", max);
    }

    #[test]
    fn test_lfo_waveforms() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_frequency(10.0);

        for waveform in [
            LfoWaveform::Sine,
            LfoWaveform::Saw,
            LfoWaveform::Triangle,
            LfoWaveform::Square,
            LfoWaveform::SampleAndHold,
        ] {
            lfo.set_waveform(waveform);
            lfo.reset();

            for _ in 0..1000 {
                let sample = lfo.process();
                assert!(sample.is_finite(), "{:?} produced invalid output", waveform);
            }
        }
    }

    #[test]
    fn test_lfo_sample_and_hold() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_waveform(LfoWaveform::SampleAndHold);
        lfo.set_frequency(10.0); // 10 Hz = 4410 samples per cycle

        // Collect samples over multiple cycles
        let mut values_per_cycle = Vec::new();
        let mut current_value = lfo.process();
        let mut last_value = current_value;

        for _ in 0..44100 {
            current_value = lfo.process();
            if (current_value - last_value).abs() > 0.001 {
                values_per_cycle.push(last_value);
            }
            last_value = current_value;
        }

        // Should have multiple distinct values (approximately 10 per second)
        assert!(values_per_cycle.len() >= 5, "S&H should produce multiple values: {}", values_per_cycle.len());
    }
}
