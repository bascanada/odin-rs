//! Oscillator modules

pub mod analog;

pub use analog::AnalogOscillator;

/// Waveform types for analog oscillator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Waveform {
    Saw = 0,
    Square = 1,
    Triangle = 2,
    Sine = 3,
}

impl Default for Waveform {
    fn default() -> Self {
        Self::Saw
    }
}

/// Common oscillator trait
pub trait Oscillator {
    /// Process one sample
    fn process(&mut self) -> f32;

    /// Set the oscillator frequency in Hz
    fn set_frequency(&mut self, freq: f32);

    /// Set the sample rate
    fn set_sample_rate(&mut self, sample_rate: f32);

    /// Reset oscillator state
    fn reset(&mut self);

    /// Randomize phase (for unison detuning)
    fn randomize_phase(&mut self);
}
