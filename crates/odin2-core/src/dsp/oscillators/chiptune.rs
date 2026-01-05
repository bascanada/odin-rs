//! Chiptune Oscillator
//!
//! An oscillator that produces classic 8-bit style sounds with an optional
//! arpeggiator for rapid note sequences, and a noise mode for percussion.

use crate::constants::WAVETABLE_LENGTH;
use crate::dsp::math::lerp;
use crate::wavetables::get_subtable;

/// Noise segment length for chip noise
const NOISE_SEGMENT_LENGTH: usize = 30;

/// Simple RNG for noise generation
struct ChipRng {
    state: u32,
}

impl ChipRng {
    fn new() -> Self {
        Self { state: 0x12345678 }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    fn next_range(&mut self, max: i32) -> i32 {
        ((self.next() as i32).abs() % max) - (max / 2)
    }
}

/// Chiptune Arpeggiator
///
/// Creates rapid note sequences by cycling through semitone offsets.
pub struct ChiptuneArpeggiator {
    /// Arpeggiator frequency (rate of note changes)
    frequency: f32,
    /// Sample rate
    sample_rate: f32,
    /// Counter for timing
    counter: f32,
    /// Current step (0, 1, or 2)
    current_step: usize,
    /// Whether arpeggiator is enabled
    enabled: bool,
    /// Whether step 3 is active (for 3-note patterns)
    step_three_active: bool,
    /// Semitone offsets for each step
    step_semitones: [i32; 3],
}

impl ChiptuneArpeggiator {
    fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 10.0,
            sample_rate,
            counter: 0.0,
            current_step: 0,
            enabled: false,
            step_three_active: false,
            step_semitones: [0, 12, 7], // Default: root, octave, fifth
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(0.1, 100.0);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn set_step_three_active(&mut self, active: bool) {
        self.step_three_active = active;
    }

    fn set_step_semitone(&mut self, step: usize, semitones: i32) {
        if step < 3 {
            self.step_semitones[step] = semitones;
        }
    }

    fn reset(&mut self) {
        self.counter = 0.0;
        self.current_step = 0;
    }

    /// Process and return current semitone offset
    fn process(&mut self) -> i32 {
        if !self.enabled {
            return 0;
        }

        let samples_per_step = self.sample_rate / self.frequency;
        self.counter += 1.0;

        if self.counter >= samples_per_step {
            self.counter = 0.0;
            let max_step = if self.step_three_active { 3 } else { 2 };
            self.current_step = (self.current_step + 1) % max_step;
        }

        self.step_semitones[self.current_step]
    }
}

/// Chiptune Oscillator
pub struct ChiptuneOscillator {
    /// Sample rate
    sample_rate: f32,
    /// One over sample rate
    one_over_sample_rate: f32,

    /// Base oscillator frequency
    frequency: f32,
    /// Read index into wavetable
    read_index: f32,
    /// Wavetable increment per sample
    wavetable_inc: f32,

    /// Current wavetable index
    wavetable_index: usize,
    /// Sub-table index (for anti-aliasing)
    sub_table_index: usize,
    /// Current wavetable pointer
    current_table: &'static [f32; WAVETABLE_LENGTH],

    /// Arpeggiator
    arpeggiator: ChiptuneArpeggiator,

    /// Whether to generate noise instead of wave
    generate_noise: bool,
    /// Last noise value
    last_noise_value: f32,
    /// Noise RNG
    noise_rng: ChipRng,

    /// IIR filter buffers for downsampling (order 9)
    xv: [f32; 10],
    yv: [f32; 10],

    /// Volume factor
    volume: f32,
}

impl ChiptuneOscillator {
    /// Create a new chiptune oscillator
    pub fn new(sample_rate: f32) -> Self {
        let frequency = 440.0;
        let wavetable_inc = WAVETABLE_LENGTH as f32 * frequency / sample_rate;
        let current_table = get_subtable(0, 0);

        Self {
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            frequency,
            read_index: 0.0,
            wavetable_inc,
            wavetable_index: 0,
            sub_table_index: 0,
            current_table,
            arpeggiator: ChiptuneArpeggiator::new(sample_rate),
            generate_noise: false,
            last_noise_value: 0.0,
            noise_rng: ChipRng::new(),
            xv: [0.0; 10],
            yv: [0.0; 10],
            volume: 1.0,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.arpeggiator.set_sample_rate(sample_rate);
        self.update_increment();
    }

    /// Set base frequency
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(0.1, 20000.0);
        self.update_increment();
        self.update_table();
    }

    /// Update wavetable increment
    fn update_increment(&mut self) {
        self.wavetable_inc = WAVETABLE_LENGTH as f32 * self.frequency * self.one_over_sample_rate;
    }

    /// Select wavetable
    pub fn select_wavetable(&mut self, index: usize) {
        self.wavetable_index = index;
        self.update_table();
    }

    /// Get the sub-table index based on frequency for anti-aliasing
    fn get_table_index(&self) -> usize {
        let ratio = self.frequency / self.sample_rate;

        if ratio < 0.001 {
            0
        } else if ratio < 0.002 {
            1
        } else if ratio < 0.004 {
            2
        } else if ratio < 0.008 {
            3
        } else if ratio < 0.016 {
            4
        } else if ratio < 0.032 {
            5
        } else if ratio < 0.064 {
            6
        } else if ratio < 0.128 {
            7
        } else {
            8
        }
    }

    /// Update wavetable pointer
    fn update_table(&mut self) {
        self.sub_table_index = self.get_table_index();
        self.current_table = get_subtable(self.wavetable_index, self.sub_table_index);
    }

    /// Enable/disable arpeggiator
    pub fn set_arp_enabled(&mut self, enabled: bool) {
        self.arpeggiator.set_enabled(enabled);
    }

    /// Set arpeggiator speed in Hz
    pub fn set_arp_speed(&mut self, speed: f32) {
        self.arpeggiator.set_frequency(speed);
    }

    /// Set arpeggiator step semitone
    pub fn set_arp_semitone(&mut self, step: usize, semitones: i32) {
        self.arpeggiator.set_step_semitone(step, semitones);
    }

    /// Enable/disable step 3 (for 3-note patterns)
    pub fn set_arp_step_three(&mut self, active: bool) {
        self.arpeggiator.set_step_three_active(active);
    }

    /// Enable/disable noise mode
    pub fn set_noise_enabled(&mut self, enabled: bool) {
        if enabled != self.generate_noise {
            self.read_index = 0.0;
            self.generate_noise = enabled;
        }
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Reset the oscillator
    pub fn reset(&mut self) {
        self.read_index = 0.0;
        self.arpeggiator.reset();
        self.xv = [0.0; 10];
        self.yv = [0.0; 10];
    }

    /// Generate chip-style noise (with 3x oversampling and lowpass filter)
    fn generate_chip_noise(&mut self) -> f32 {
        // Advance read index
        self.read_index += self.wavetable_inc;

        // Generate new random value when segment ends
        if self.read_index > (NOISE_SEGMENT_LENGTH * 3) as f32 {
            self.read_index = 0.0;
            self.last_noise_value = self.noise_rng.next_range(16) as f32 * 0.125;
        }

        // Apply 9th order IIR lowpass filter for downsampling (3x oversampling)
        // Butterworth LP, order 9, SR 132300, corner 40000
        for _ in 0..3 {
            // Shift buffers
            for i in 0..9 {
                self.xv[i] = self.xv[i + 1];
                self.yv[i] = self.yv[i + 1];
            }

            self.xv[9] = self.last_noise_value * 0.019966841051093;
            self.yv[9] = (self.xv[0] + self.xv[9])
                + 9.0 * (self.xv[1] + self.xv[8])
                + 36.0 * (self.xv[2] + self.xv[7])
                + 84.0 * (self.xv[3] + self.xv[6])
                + 126.0 * (self.xv[4] + self.xv[5])
                + (-0.0003977153 * self.yv[0])
                + (-0.0064474617 * self.yv[1])
                + (-0.0476997403 * self.yv[2])
                + (-0.2185829743 * self.yv[3])
                + (-0.6649234123 * self.yv[4])
                + (-1.4773657709 * self.yv[5])
                + (-2.2721421641 * self.yv[6])
                + (-2.6598673212 * self.yv[7])
                + (-1.8755960587 * self.yv[8]);
        }

        self.yv[9]
    }

    /// Read from wavetable with linear interpolation
    fn read_wavetable(&mut self) -> f32 {
        let read_index_trunc = self.read_index as usize;
        let frac = self.read_index - read_index_trunc as f32;
        let read_index_next = if read_index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            read_index_trunc + 1
        };

        let output = lerp(
            self.current_table[read_index_trunc],
            self.current_table[read_index_next],
            frac,
        );

        // Increment read index
        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        output
    }

    /// Process one sample
    pub fn process(&mut self) -> f32 {
        // Get arpeggiator pitch offset
        let arp_semitones = self.arpeggiator.process();

        // Apply arpeggiator pitch shift if active
        if arp_semitones != 0 {
            let pitch_mult = crate::dsp::math::pitch_shift_multiplier(arp_semitones as f32);
            self.wavetable_inc =
                WAVETABLE_LENGTH as f32 * self.frequency * pitch_mult * self.one_over_sample_rate;
            self.update_table();
        }

        let output = if self.generate_noise {
            self.generate_chip_noise()
        } else {
            self.read_wavetable()
        };

        output * self.volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chiptune_basic() {
        let mut osc = ChiptuneOscillator::new(44100.0);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let output = osc.process();
            assert!(output.is_finite(), "Chiptune oscillator produced invalid output");
        }
    }

    #[test]
    fn test_chiptune_noise() {
        let mut osc = ChiptuneOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_noise_enabled(true);

        let mut has_variation = false;
        let mut prev = 0.0;

        for _ in 0..1000 {
            let output = osc.process();
            assert!(output.is_finite());
            if (output - prev).abs() > 0.001 {
                has_variation = true;
            }
            prev = output;
        }

        assert!(has_variation, "Noise mode should create variation");
    }

    #[test]
    fn test_chiptune_arpeggiator() {
        let mut osc = ChiptuneOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_arp_enabled(true);
        osc.set_arp_speed(20.0); // Fast arp
        osc.set_arp_semitone(0, 0);
        osc.set_arp_semitone(1, 12); // Octave up
        osc.set_arp_semitone(2, 7); // Fifth

        // Process enough samples for arpeggiator to cycle
        for _ in 0..10000 {
            let output = osc.process();
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_chiptune_wavetables() {
        let mut osc = ChiptuneOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Test different wavetables
        for wt in 0..10 {
            osc.reset();
            osc.select_wavetable(wt);

            for _ in 0..1000 {
                let output = osc.process();
                assert!(output.is_finite(), "Wavetable {} produced invalid output", wt);
            }
        }
    }

    #[test]
    fn test_chiptune_step_three() {
        let mut osc = ChiptuneOscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_arp_enabled(true);
        osc.set_arp_speed(30.0);
        osc.set_arp_step_three(true);
        osc.set_arp_semitone(0, 0);
        osc.set_arp_semitone(1, 4); // Major third
        osc.set_arp_semitone(2, 7); // Fifth

        for _ in 0..10000 {
            let output = osc.process();
            assert!(output.is_finite());
        }
    }
}
