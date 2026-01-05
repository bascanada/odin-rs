//! Vector Oscillator
//!
//! A 2D wavetable oscillator that blends between 4 wavetables based on X/Y position.
//! Creates smooth morphing between different timbres using bilinear interpolation.

use crate::constants::{NUMBER_OF_WAVETABLES, WAVETABLE_LENGTH};
use crate::dsp::math::lerp;
use crate::wavetables::get_subtable;

/// Number of corner points in the vector pad
const VECTOR_EDGES: usize = 4;

/// Vector Oscillator
///
/// Uses a 2D pad to blend between 4 different wavetables:
/// ```text
/// 1---2
/// |   |
/// 0---3
/// ```
/// X controls horizontal blend, Y controls vertical blend.
pub struct VectorOscillator {
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

    /// Wavetable indices for each corner
    wavetable_indices: [usize; VECTOR_EDGES],
    /// Sub-table index (for anti-aliasing)
    sub_table_index: usize,
    /// Current wavetable pointers for each corner
    current_tables: [&'static [f32; WAVETABLE_LENGTH]; VECTOR_EDGES],

    /// X position (0.0 to 1.0)
    xy_pad_x: f32,
    /// Y position (0.0 to 1.0)
    xy_pad_y: f32,
    /// Smoothed X position
    xy_pad_x_smooth: f32,
    /// Smoothed Y position
    xy_pad_y_smooth: f32,
}

impl VectorOscillator {
    /// Create a new vector oscillator
    pub fn new(sample_rate: f32) -> Self {
        let frequency = 440.0;
        let wavetable_inc = WAVETABLE_LENGTH as f32 * frequency / sample_rate;

        // Default to sine wave for all corners (first wavetable, first subtable)
        let default_table = get_subtable(0, 0);

        Self {
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            frequency,
            read_index: 0.0,
            wavetable_inc,
            wavetable_indices: [0; VECTOR_EDGES],
            sub_table_index: 0,
            current_tables: [default_table; VECTOR_EDGES],
            xy_pad_x: 0.0,
            xy_pad_y: 0.0,
            xy_pad_x_smooth: 0.0,
            xy_pad_y_smooth: 0.0,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        self.update_increment();
    }

    /// Set oscillator frequency
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(0.1, 20000.0);
        self.update_increment();
        self.update_tables();
    }

    /// Update wavetable increment
    fn update_increment(&mut self) {
        self.wavetable_inc = WAVETABLE_LENGTH as f32 * self.frequency * self.one_over_sample_rate;
    }

    /// Set X position (0.0 to 1.0)
    pub fn set_x(&mut self, x: f32) {
        self.xy_pad_x = x.clamp(0.0, 1.0);
    }

    /// Set Y position (0.0 to 1.0)
    pub fn set_y(&mut self, y: f32) {
        self.xy_pad_y = y.clamp(0.0, 1.0);
    }

    /// Set XY position
    pub fn set_xy(&mut self, x: f32, y: f32) {
        self.set_x(x);
        self.set_y(y);
    }

    /// Select wavetable for a corner (0-3)
    /// Corner layout:
    /// ```text
    /// 1---2
    /// |   |
    /// 0---3
    /// ```
    pub fn select_wavetable(&mut self, corner: usize, wavetable_index: usize) {
        if corner < VECTOR_EDGES && wavetable_index < NUMBER_OF_WAVETABLES {
            self.wavetable_indices[corner] = wavetable_index;
            self.update_tables();
        }
    }

    /// Get the sub-table index based on frequency for anti-aliasing
    fn get_table_index(&self) -> usize {
        // Calculate which sub-table to use based on frequency
        // Higher frequencies use higher sub-tables with fewer harmonics
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

    /// Update wavetable pointers
    fn update_tables(&mut self) {
        self.sub_table_index = self.get_table_index();
        for i in 0..VECTOR_EDGES {
            self.current_tables[i] = get_subtable(self.wavetable_indices[i], self.sub_table_index);
        }
    }

    /// Reset the oscillator
    pub fn reset(&mut self) {
        self.read_index = 0.0;
        self.xy_pad_x_smooth = self.xy_pad_x;
        self.xy_pad_y_smooth = self.xy_pad_y;
    }

    /// Randomize phase
    pub fn randomize_phase(&mut self) {
        // Simple LCG for deterministic but varied phases
        static mut SEED: u32 = 12345;
        unsafe {
            SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
            self.read_index = (SEED as f32 / u32::MAX as f32) * WAVETABLE_LENGTH as f32;
        }
    }

    /// Read from a wavetable with linear interpolation
    fn read_table(&self, table: &[f32; WAVETABLE_LENGTH]) -> f32 {
        let read_index_trunc = self.read_index as usize;
        let frac = self.read_index - read_index_trunc as f32;
        let read_index_next = if read_index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            read_index_trunc + 1
        };

        lerp(table[read_index_trunc], table[read_index_next], frac)
    }

    /// Process one sample
    pub fn process(&mut self) -> f32 {
        // Smooth XY controls
        self.xy_pad_x_smooth += (self.xy_pad_x - self.xy_pad_x_smooth) * 0.001;
        self.xy_pad_y_smooth += (self.xy_pad_y - self.xy_pad_y_smooth) * 0.001;

        // Read from all 4 corners
        let output: [f32; VECTOR_EDGES] = [
            self.read_table(self.current_tables[0]),
            self.read_table(self.current_tables[1]),
            self.read_table(self.current_tables[2]),
            self.read_table(self.current_tables[3]),
        ];

        let x = self.xy_pad_x_smooth;
        let y = self.xy_pad_y_smooth;

        // Bilinear interpolation
        // Layout:
        // 1---2
        // |   |
        // 0---3

        // Bottom edge (0 to 3)
        let bottom = (1.0 - x) * output[0] + x * output[3];
        // Top edge (1 to 2)
        let top = (1.0 - x) * output[1] + x * output[2];

        // Increment read index
        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        // Final blend between bottom and top
        (1.0 - y) * bottom + y * top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_basic() {
        let mut osc = VectorOscillator::new(44100.0);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let output = osc.process();
            assert!(output.is_finite(), "Vector oscillator produced invalid output");
            assert!(output >= -1.5 && output <= 1.5, "Output out of range: {}", output);
        }
    }

    #[test]
    fn test_vector_corners() {
        let mut osc = VectorOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Test each corner position
        let positions = [
            (0.0, 0.0), // Corner 0
            (0.0, 1.0), // Corner 1
            (1.0, 1.0), // Corner 2
            (1.0, 0.0), // Corner 3
        ];

        for (x, y) in positions {
            osc.reset();
            osc.set_xy(x, y);
            // Process enough for smoothing to settle
            for _ in 0..10000 {
                let output = osc.process();
                assert!(output.is_finite());
            }
        }
    }

    #[test]
    fn test_vector_morphing() {
        let mut osc = VectorOscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Set different wavetables for corners
        osc.select_wavetable(0, 0); // Sine
        osc.select_wavetable(1, 1); // Different
        osc.select_wavetable(2, 2); // Different
        osc.select_wavetable(3, 3); // Different

        // Morph through positions
        for i in 0..1000 {
            let t = i as f32 / 1000.0;
            osc.set_xy(t, t);
            let output = osc.process();
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_vector_frequencies() {
        let mut osc = VectorOscillator::new(44100.0);
        osc.set_xy(0.5, 0.5);

        for freq in [100.0, 440.0, 1000.0, 5000.0, 10000.0] {
            osc.reset();
            osc.set_frequency(freq);

            for _ in 0..1000 {
                let output = osc.process();
                assert!(output.is_finite(), "Vector at {}Hz produced invalid output", freq);
            }
        }
    }
}
