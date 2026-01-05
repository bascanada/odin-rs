//! Filter modules

pub mod ladder;
pub mod va_one_pole;

pub use ladder::LadderFilter;
pub use va_one_pole::VAOnePoleFilter;

/// Filter type for ladder filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LadderFilterType {
    LP4 = 0,
    LP2 = 1,
    BP4 = 2,
    BP2 = 3,
    HP4 = 4,
    HP2 = 5,
}

impl Default for LadderFilterType {
    fn default() -> Self {
        Self::LP4
    }
}

/// Common filter trait
pub trait Filter {
    /// Process one sample
    fn process(&mut self, input: f32) -> f32;

    /// Set cutoff frequency in Hz
    fn set_cutoff(&mut self, cutoff: f32);

    /// Set resonance (0.0 to 1.0 typically)
    fn set_resonance(&mut self, resonance: f32);

    /// Set sample rate
    fn set_sample_rate(&mut self, sample_rate: f32);

    /// Reset filter state
    fn reset(&mut self);
}
