//! Wavetable 2D Oscillator
//!
//! A wavetable oscillator with position-based morphing between 4 sub-wavetables.
//! Supports 35 preset 2D wavetables, each containing 4 wavetables that can be
//! morphed between using the position parameter.

use super::Oscillator;
use crate::constants::*;
use crate::dsp::math::lerp;
use crate::wavetables;

/// Number of 2D wavetable presets
pub const NUMBER_OF_WAVETABLES_2D: usize = 35;

/// Number of sub-tables per 2D wavetable
pub const TABLES_PER_2D_WT: usize = 4;

/// 2D Wavetable preset definition
/// Each preset contains 4 wavetable indices that can be morphed between
#[derive(Clone, Copy)]
pub struct Wavetable2DPreset {
    /// Name of the preset
    pub name: &'static str,
    /// Wavetable indices for the 4 sub-tables
    pub tables: [usize; TABLES_PER_2D_WT],
}

/// All 35 2D wavetable presets
/// Based on the original Odin 2 C++ implementation
pub const WAVETABLE_2D_PRESETS: [Wavetable2DPreset; NUMBER_OF_WAVETABLES_2D] = [
    // 0: Basic - Saw -> Square -> Triangle -> Sine
    Wavetable2DPreset {
        name: "Basic",
        tables: [1, 3, 2, 0], // FatSaw, Square, Triangle, Sine
    },
    // 1: Birds
    Wavetable2DPreset {
        name: "Birds",
        tables: [4, 5, 6, 7],
    },
    // 2: BagPipe
    Wavetable2DPreset {
        name: "BagPipe",
        tables: [8, 9, 10, 11],
    },
    // 3: Glass
    Wavetable2DPreset {
        name: "Glass",
        tables: [12, 13, 14, 15],
    },
    // 4: FMSynth
    Wavetable2DPreset {
        name: "FMSynth",
        tables: [16, 17, 18, 19],
    },
    // 5: BrokenSine
    Wavetable2DPreset {
        name: "BrokenSine",
        tables: [20, 21, 22, 23],
    },
    // 6: Skyline
    Wavetable2DPreset {
        name: "Skyline",
        tables: [24, 25, 26, 27],
    },
    // 7: Perlin
    Wavetable2DPreset {
        name: "Perlin",
        tables: [28, 29, 30, 31],
    },
    // 8: Rectangular
    Wavetable2DPreset {
        name: "Rectangular",
        tables: [32, 33, 34, 35],
    },
    // 9: Bitreduced
    Wavetable2DPreset {
        name: "Bitreduced",
        tables: [36, 37, 38, 39],
    },
    // 10: Strings
    Wavetable2DPreset {
        name: "Strings",
        tables: [40, 41, 42, 43],
    },
    // 11: Piano
    Wavetable2DPreset {
        name: "Piano",
        tables: [44, 45, 46, 47],
    },
    // 12: Organ
    Wavetable2DPreset {
        name: "Organ",
        tables: [48, 49, 50, 51],
    },
    // 13: Oboe
    Wavetable2DPreset {
        name: "Oboe",
        tables: [52, 53, 54, 55],
    },
    // 14: Trumpet
    Wavetable2DPreset {
        name: "Trumpet",
        tables: [56, 57, 58, 59],
    },
    // 15: Legacy1
    Wavetable2DPreset {
        name: "Legacy1",
        tables: [60, 61, 62, 63],
    },
    // 16: Legacy2
    Wavetable2DPreset {
        name: "Legacy2",
        tables: [64, 65, 66, 67],
    },
    // 17: Legacy3
    Wavetable2DPreset {
        name: "Legacy3",
        tables: [68, 69, 70, 71],
    },
    // 18: Legacy4
    Wavetable2DPreset {
        name: "Legacy4",
        tables: [72, 73, 74, 75],
    },
    // 19: Voice1
    Wavetable2DPreset {
        name: "Voice1",
        tables: [76, 77, 78, 79],
    },
    // 20: Voice2
    Wavetable2DPreset {
        name: "Voice2",
        tables: [80, 81, 82, 83],
    },
    // 21: Voice3
    Wavetable2DPreset {
        name: "Voice3",
        tables: [84, 85, 86, 87],
    },
    // 22: Voice4
    Wavetable2DPreset {
        name: "Voice4",
        tables: [88, 89, 90, 91],
    },
    // 23: Additive1
    Wavetable2DPreset {
        name: "Additive1",
        tables: [92, 93, 94, 95],
    },
    // 24: Additive2
    Wavetable2DPreset {
        name: "Additive2",
        tables: [96, 97, 98, 99],
    },
    // 25: Additive3
    Wavetable2DPreset {
        name: "Additive3",
        tables: [100, 101, 102, 103],
    },
    // 26: Additive4
    Wavetable2DPreset {
        name: "Additive4",
        tables: [104, 105, 106, 107],
    },
    // 27: Harmonics1
    Wavetable2DPreset {
        name: "Harmonics1",
        tables: [108, 109, 110, 111],
    },
    // 28: Harmonics2
    Wavetable2DPreset {
        name: "Harmonics2",
        tables: [112, 113, 114, 115],
    },
    // 29: Harmonics3
    Wavetable2DPreset {
        name: "Harmonics3",
        tables: [116, 117, 118, 119],
    },
    // 30: Harmonics4
    Wavetable2DPreset {
        name: "Harmonics4",
        tables: [120, 121, 122, 123],
    },
    // 31: FatSaw1
    Wavetable2DPreset {
        name: "FatSaw1",
        tables: [124, 125, 126, 127],
    },
    // 32: FatSaw2
    Wavetable2DPreset {
        name: "FatSaw2",
        tables: [128, 129, 130, 131],
    },
    // 33: ChipMutate1
    Wavetable2DPreset {
        name: "ChipMutate1",
        tables: [132, 133, 134, 135],
    },
    // 34: ChipMutate2
    Wavetable2DPreset {
        name: "ChipMutate2",
        tables: [136, 137, 138, 139],
    },
];

/// Minor third frequency ratios for subtable selection
const TABLE_FREQUENCIES: [f32; SUBTABLES_PER_WAVETABLE] = [
    25.18, 29.94, 35.61, 42.35, 50.37, 59.91, 71.25, 84.74, 100.77, 119.85, 142.54,
    169.54, 201.60, 239.77, 285.15, 339.16, 403.33, 479.68, 570.47, 678.51, 806.89,
    959.62, 1141.24, 1357.37, 1614.17, 1919.67, 2282.98, 2715.30, 3228.98, 3840.07,
    4566.78, 5431.49, 6458.91,
];

/// 2D Wavetable Oscillator
///
/// Morphs between 4 wavetables based on a position parameter (0.0 to 1.0).
/// Position 0.0 = table 0
/// Position 0.333 = table 1
/// Position 0.666 = table 2
/// Position 1.0 = table 3
pub struct WavetableOsc2D {
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

    /// Current 2D wavetable preset index (0-34)
    preset_index: usize,

    /// Current subtable index (0-32, selected based on frequency)
    subtable_index: usize,

    /// Position in 2D space (0.0 to 1.0)
    position: f32,

    /// Smoothed position for click-free morphing
    position_smooth: f32,

    /// Position modulation amount
    pos_mod_amount: f32,

    /// Cached pointers to the 4 sub-tables (at current subtable_index)
    current_tables: [&'static [f32; WAVETABLE_LENGTH]; TABLES_PER_2D_WT],
}

impl WavetableOsc2D {
    /// Create a new 2D wavetable oscillator
    pub fn new(sample_rate: f32) -> Self {
        let preset = &WAVETABLE_2D_PRESETS[0];
        let subtable_index = 0;

        Self {
            read_index: 0.0,
            wavetable_inc: 0.0,
            frequency: OSC_FREQ_DEFAULT,
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            preset_index: 0,
            subtable_index,
            position: 0.0,
            position_smooth: 0.0,
            pos_mod_amount: 0.0,
            current_tables: [
                wavetables::get_subtable(preset.tables[0], subtable_index),
                wavetables::get_subtable(preset.tables[1], subtable_index),
                wavetables::get_subtable(preset.tables[2], subtable_index),
                wavetables::get_subtable(preset.tables[3], subtable_index),
            ],
        }
    }

    /// Set the 2D wavetable preset by index (0-34)
    pub fn set_preset(&mut self, index: usize) {
        self.preset_index = index.min(NUMBER_OF_WAVETABLES_2D - 1);
        self.update_tables();
    }

    /// Set the 2D wavetable preset by name
    pub fn set_preset_by_name(&mut self, name: &str) -> bool {
        for (i, preset) in WAVETABLE_2D_PRESETS.iter().enumerate() {
            if preset.name == name {
                self.set_preset(i);
                return true;
            }
        }
        false
    }

    /// Get the current preset index
    pub fn preset_index(&self) -> usize {
        self.preset_index
    }

    /// Get the current preset name
    pub fn preset_name(&self) -> &'static str {
        WAVETABLE_2D_PRESETS[self.preset_index].name
    }

    /// Set the position (0.0 to 1.0) for morphing between the 4 tables
    pub fn set_position(&mut self, position: f32) {
        self.position = position.clamp(0.0, 1.0);
    }

    /// Get the current position
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Set position modulation amount
    pub fn set_pos_mod_amount(&mut self, amount: f32) {
        self.pos_mod_amount = amount;
    }

    /// Update the cached table pointers based on preset and frequency
    fn update_tables(&mut self) {
        let preset = &WAVETABLE_2D_PRESETS[self.preset_index];
        self.current_tables = [
            wavetables::get_subtable(preset.tables[0], self.subtable_index),
            wavetables::get_subtable(preset.tables[1], self.subtable_index),
            wavetables::get_subtable(preset.tables[2], self.subtable_index),
            wavetables::get_subtable(preset.tables[3], self.subtable_index),
        ];
    }

    /// Update subtable based on current frequency
    fn update_subtable(&mut self) {
        let mut new_subtable = 0;
        for (i, &freq) in TABLE_FREQUENCIES.iter().enumerate() {
            if self.frequency < freq {
                break;
            }
            new_subtable = i;
        }

        if new_subtable != self.subtable_index {
            self.subtable_index = new_subtable;
            self.update_tables();
        }
    }

    /// Get the left table index, right table index, and interpolation value
    /// based on the current position
    #[inline]
    fn get_table_indices_and_interpolation(&self, position: f32) -> (usize, usize, f32) {
        if position < 0.333333333333 {
            (0, 1, position * 3.0)
        } else if position < 0.666666666 {
            (1, 2, (position - 0.333333333) * 3.0)
        } else {
            (2, 3, (position - 0.666666666) * 3.0)
        }
    }

    /// Read from wavetable with linear interpolation
    #[inline]
    fn read_wavetable(&self, table: &[f32; WAVETABLE_LENGTH]) -> f32 {
        let index_trunc = self.read_index as usize;
        let fractional = self.read_index - index_trunc as f32;
        let index_next = if index_trunc + 1 >= WAVETABLE_LENGTH {
            0
        } else {
            index_trunc + 1
        };

        lerp(table[index_trunc], table[index_next], fractional)
    }

    /// Process one sample with position morphing
    fn do_wavetable_2d(&mut self, pos_mod: f32) -> f32 {
        // Smooth position changes
        self.position_smooth += (self.position - self.position_smooth) * 0.001;

        // Apply modulation
        let position_modded = (self.position_smooth + pos_mod + self.pos_mod_amount).clamp(0.0, 1.0);

        // Get interpolation parameters
        let (left_table, right_table, interpolation) =
            self.get_table_indices_and_interpolation(position_modded);

        // Read from both tables
        let output_left = self.read_wavetable(self.current_tables[left_table]);
        let output_right = self.read_wavetable(self.current_tables[right_table]);

        // Advance read index
        self.read_index += self.wavetable_inc;
        while self.read_index >= WAVETABLE_LENGTH as f32 {
            self.read_index -= WAVETABLE_LENGTH as f32;
        }

        // Linear interpolation between tables
        lerp(output_left, output_right, interpolation)
    }

    /// Process with external position modulation
    pub fn process_with_mod(&mut self, pos_mod: f32) -> f32 {
        self.do_wavetable_2d(pos_mod)
    }
}

impl Oscillator for WavetableOsc2D {
    fn process(&mut self) -> f32 {
        self.do_wavetable_2d(0.0)
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
        self.position_smooth = self.position;
    }

    fn randomize_phase(&mut self) {
        self.read_index = (self.read_index * 12345.6789) % (WAVETABLE_LENGTH as f32);
    }
}

impl Default for WavetableOsc2D {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavetable_2d_basic() {
        let mut osc = WavetableOsc2D::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_position(0.0);

        // Process some samples
        for _ in 0..1000 {
            let sample = osc.process();
            assert!(sample.is_finite(), "WavetableOsc2D produced invalid output");
            assert!(
                sample >= -1.1 && sample <= 1.1,
                "WavetableOsc2D output out of range: {}",
                sample
            );
        }
    }

    #[test]
    fn test_wavetable_2d_position_sweep() {
        let mut osc = WavetableOsc2D::new(44100.0);
        osc.set_frequency(440.0);

        // Sweep through positions
        for i in 0..1000 {
            let position = i as f32 / 1000.0;
            osc.set_position(position);
            let sample = osc.process();
            assert!(sample.is_finite(), "Invalid output at position {}", position);
        }
    }

    #[test]
    fn test_wavetable_2d_presets() {
        let mut osc = WavetableOsc2D::new(44100.0);
        osc.set_frequency(440.0);

        // Test all presets
        for i in 0..NUMBER_OF_WAVETABLES_2D {
            osc.set_preset(i);
            osc.set_position(0.5);

            for _ in 0..100 {
                let sample = osc.process();
                assert!(
                    sample.is_finite(),
                    "Preset {} produced invalid output",
                    WAVETABLE_2D_PRESETS[i].name
                );
            }
        }
    }

    #[test]
    fn test_wavetable_2d_preset_by_name() {
        let mut osc = WavetableOsc2D::new(44100.0);

        assert!(osc.set_preset_by_name("Basic"));
        assert_eq!(osc.preset_name(), "Basic");

        assert!(osc.set_preset_by_name("Glass"));
        assert_eq!(osc.preset_name(), "Glass");

        // Non-existent preset should return false
        assert!(!osc.set_preset_by_name("NonExistent"));
    }

    #[test]
    fn test_wavetable_2d_interpolation() {
        let osc = WavetableOsc2D::new(44100.0);

        // Test position 0.0 -> tables 0,1 interpolation 0.0
        let (left, right, interp) = osc.get_table_indices_and_interpolation(0.0);
        assert_eq!(left, 0);
        assert_eq!(right, 1);
        assert!((interp - 0.0).abs() < 0.001);

        // Test position 0.333 -> tables 0,1 interpolation ~1.0
        let (left, right, interp) = osc.get_table_indices_and_interpolation(0.333);
        assert_eq!(left, 0);
        assert_eq!(right, 1);
        assert!((interp - 0.999).abs() < 0.01);

        // Test position 0.5 -> tables 1,2 interpolation ~0.5
        let (left, right, interp) = osc.get_table_indices_and_interpolation(0.5);
        assert_eq!(left, 1);
        assert_eq!(right, 2);
        assert!((interp - 0.5).abs() < 0.1);

        // Test position 1.0 -> tables 2,3 interpolation 1.0
        let (left, right, interp) = osc.get_table_indices_and_interpolation(1.0);
        assert_eq!(left, 2);
        assert_eq!(right, 3);
        assert!((interp - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_wavetable_2d_frequency_subtable() {
        let mut osc = WavetableOsc2D::new(44100.0);

        osc.set_frequency(100.0);
        let low_subtable = osc.subtable_index;

        osc.set_frequency(5000.0);
        let high_subtable = osc.subtable_index;

        // Higher frequency should use higher subtable
        assert!(
            high_subtable > low_subtable,
            "Low freq subtable: {}, High freq subtable: {}",
            low_subtable,
            high_subtable
        );
    }

    #[test]
    fn test_wavetable_2d_modulation() {
        let mut osc = WavetableOsc2D::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_position(0.0);

        // Process with modulation
        for i in 0..1000 {
            let mod_value = libm::sinf(i as f32 * 0.01) * 0.5 + 0.5;
            let sample = osc.process_with_mod(mod_value);
            assert!(sample.is_finite(), "Invalid output with modulation");
        }
    }
}
