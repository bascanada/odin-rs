//! Modulation Matrix
//!
//! The modulation matrix routes modulation sources to destinations.
//! Unlike the C++ implementation which uses pointers, this uses an index-based system.

use crate::constants::*;

/// Modulation source identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModSource {
    None = 0,
    Lfo1 = 1,
    Lfo2 = 2,
    Lfo3 = 3,
    Env1 = 4,  // Amp envelope
    Env2 = 5,  // Filter envelope
    Env3 = 6,  // Mod envelope
    Velocity = 7,
    Note = 8,
    ModWheel = 9,
    PitchWheel = 10,
    Aftertouch = 11,
    Random = 12,
    Constant = 13,
}

/// Modulation destination identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModDest {
    None = 0,
    // Oscillator 1
    Osc1Pitch = 1,
    Osc1Amp = 2,
    Osc1Pwm = 3,
    // Oscillator 2
    Osc2Pitch = 4,
    Osc2Amp = 5,
    Osc2Pwm = 6,
    // Oscillator 3
    Osc3Pitch = 7,
    Osc3Amp = 8,
    Osc3Pwm = 9,
    // Filter 1
    Filter1Cutoff = 10,
    Filter1Resonance = 11,
    // Filter 2
    Filter2Cutoff = 12,
    Filter2Resonance = 13,
    // Global
    Volume = 14,
    Pan = 15,
}

/// A single row in the modulation matrix
#[derive(Debug, Clone, Copy)]
pub struct ModMatrixRow {
    /// Source modulator
    pub source: ModSource,
    /// First destination
    pub dest1: ModDest,
    /// Amount for first destination (-1.0 to 1.0)
    pub amount1: f32,
    /// Second destination
    pub dest2: ModDest,
    /// Amount for second destination (-1.0 to 1.0)
    pub amount2: f32,
}

impl Default for ModMatrixRow {
    fn default() -> Self {
        Self {
            source: ModSource::None,
            dest1: ModDest::None,
            amount1: 0.0,
            dest2: ModDest::None,
            amount2: 0.0,
        }
    }
}

/// Frame of modulation values computed per sample/block
#[derive(Debug, Clone)]
pub struct ModFrame {
    /// Source values (LFOs, envelopes, etc.)
    pub sources: [f32; 16],
    /// Computed destination modulation values
    pub destinations: [f32; 32],
}

impl Default for ModFrame {
    fn default() -> Self {
        Self {
            sources: [0.0; 16],
            destinations: [0.0; 32],
        }
    }
}

impl ModFrame {
    /// Reset all values to zero
    pub fn reset(&mut self) {
        self.sources.fill(0.0);
        self.destinations.fill(0.0);
    }

    /// Set a source value
    pub fn set_source(&mut self, source: ModSource, value: f32) {
        self.sources[source as usize] = value;
    }

    /// Get a destination modulation value
    pub fn get_destination(&self, dest: ModDest) -> f32 {
        self.destinations[dest as usize]
    }
}

/// The modulation matrix
pub struct ModMatrix {
    /// Matrix rows
    pub rows: [ModMatrixRow; MOD_MATRIX_ROWS],
}

impl Default for ModMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl ModMatrix {
    /// Create a new modulation matrix
    pub fn new() -> Self {
        Self {
            rows: [ModMatrixRow::default(); MOD_MATRIX_ROWS],
        }
    }

    /// Apply modulation to compute destination values
    pub fn apply(&self, frame: &mut ModFrame) {
        // Reset destinations
        frame.destinations.fill(0.0);

        // Apply each row
        for row in &self.rows {
            if row.source == ModSource::None {
                continue;
            }

            let source_value = frame.sources[row.source as usize];

            // Apply to destination 1
            if row.dest1 != ModDest::None {
                frame.destinations[row.dest1 as usize] += source_value * row.amount1;
            }

            // Apply to destination 2
            if row.dest2 != ModDest::None {
                frame.destinations[row.dest2 as usize] += source_value * row.amount2;
            }
        }
    }

    /// Set a row configuration
    pub fn set_row(
        &mut self,
        row_index: usize,
        source: ModSource,
        dest1: ModDest,
        amount1: f32,
        dest2: ModDest,
        amount2: f32,
    ) {
        if row_index < MOD_MATRIX_ROWS {
            self.rows[row_index] = ModMatrixRow {
                source,
                dest1,
                amount1,
                dest2,
                amount2,
            };
        }
    }
}
