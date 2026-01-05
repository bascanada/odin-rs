//! Analog Oscillator
//!
//! Virtual analog oscillator with saw, square, triangle, and sine waveforms.
//! Uses PolyBLEP anti-aliasing for saw and square waves.

use super::{Oscillator, Waveform};
use crate::dsp::math::*;
use crate::constants::*;

/// Analog oscillator with classic waveforms
pub struct AnalogOscillator {
    /// Current phase (0.0 to 1.0)
    phase: f32,

    /// Phase increment per sample
    phase_inc: f32,

    /// Current frequency in Hz
    frequency: f32,

    /// Sample rate
    sample_rate: f32,

    /// One over sample rate (cached)
    one_over_sample_rate: f32,

    /// Current waveform
    waveform: Waveform,

    /// Pulse width for square wave (0.0 to 1.0, default 0.5)
    pulse_width: f32,

    /// PWM modulation amount (-1.0 to 1.0)
    pwm_mod: f32,

    /// Drift amount (for analog feel)
    drift: f32,

    /// Drift noise value (smoothed)
    drift_noise: f32,
}

impl AnalogOscillator {
    /// Create a new analog oscillator
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
            frequency: OSC_FREQ_DEFAULT,
            sample_rate,
            one_over_sample_rate: 1.0 / sample_rate,
            waveform: Waveform::Saw,
            pulse_width: 0.5,
            pwm_mod: 0.0,
            drift: 0.0,
            drift_noise: 0.0,
        }
    }

    /// Set the waveform
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set pulse width (0.02 to 0.98)
    pub fn set_pulse_width(&mut self, pw: f32) {
        self.pulse_width = pw.clamp(0.02, 0.98);
    }

    /// Set PWM modulation
    pub fn set_pwm_mod(&mut self, mod_amount: f32) {
        self.pwm_mod = mod_amount.clamp(-1.0, 1.0);
    }

    /// Set drift amount
    pub fn set_drift(&mut self, drift: f32) {
        self.drift = drift.clamp(0.0, 1.0);
    }

    /// PolyBLEP function for anti-aliasing
    ///
    /// Reduces aliasing at discontinuities in saw and square waves.
    #[inline]
    fn poly_blep(&self, t: f32) -> f32 {
        let dt = self.phase_inc;

        if t < dt {
            // Just after discontinuity
            let t = t / dt;
            2.0 * t - t * t - 1.0
        } else if t > 1.0 - dt {
            // Just before discontinuity
            let t = (t - 1.0) / dt;
            t * t + 2.0 * t + 1.0
        } else {
            0.0
        }
    }

    /// Generate saw wave with PolyBLEP
    #[inline]
    fn generate_saw(&self) -> f32 {
        // Naive saw: 2 * phase - 1 (range -1 to 1)
        let mut saw = 2.0 * self.phase - 1.0;

        // Apply PolyBLEP at the discontinuity (phase = 0/1)
        saw -= self.poly_blep(self.phase);

        saw
    }

    /// Generate square wave with PolyBLEP
    #[inline]
    fn generate_square(&self) -> f32 {
        // Calculate effective pulse width with modulation
        let pw = (self.pulse_width + self.pwm_mod * 0.5).clamp(0.02, 0.98);

        // Naive square
        let mut square = if self.phase < pw { 1.0 } else { -1.0 };

        // Apply PolyBLEP at both edges
        square += self.poly_blep(self.phase);
        square -= self.poly_blep((self.phase + (1.0 - pw)) % 1.0);

        square * 0.5 // Reduce volume to match other waveforms
    }

    /// Generate triangle wave
    #[inline]
    fn generate_triangle(&self) -> f32 {
        // Triangle from integrated square
        // More efficient: direct calculation
        let phase = self.phase;
        if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        }
    }

    /// Generate sine wave
    #[inline]
    fn generate_sine(&self) -> f32 {
        fast_sin(self.phase * TWO_PI)
    }

    /// Generate drift noise for analog feel
    fn generate_drift(&mut self) -> f32 {
        // Simple random walk for drift
        // In production, use proper noise generator
        let noise = self.drift_noise * 0.9999 + libm::sinf(self.phase * 1000.0) * 0.0001;
        self.drift_noise = noise;
        noise * self.drift * 0.2 // 0.2 semitones max drift
    }
}

impl Oscillator for AnalogOscillator {
    fn process(&mut self) -> f32 {
        // Generate output based on waveform
        let output = match self.waveform {
            Waveform::Saw => self.generate_saw(),
            Waveform::Square => self.generate_square(),
            Waveform::Triangle => self.generate_triangle(),
            Waveform::Sine => self.generate_sine(),
        };

        // Advance phase
        self.phase += self.phase_inc;

        // Wrap phase
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    fn set_frequency(&mut self, freq: f32) {
        // Apply drift if enabled
        let drift_mod = if self.drift > 0.0 {
            self.generate_drift()
        } else {
            0.0
        };

        let freq_with_drift = freq * pitch_shift_multiplier(drift_mod);
        self.frequency = freq_with_drift.clamp(OSC_FREQ_MIN, OSC_FREQ_MAX);
        self.phase_inc = self.frequency * self.one_over_sample_rate;
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.one_over_sample_rate = 1.0 / sample_rate;
        // Recalculate phase increment
        self.phase_inc = self.frequency * self.one_over_sample_rate;
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.drift_noise = 0.0;
    }

    fn randomize_phase(&mut self) {
        // Simple pseudo-random based on current phase and frequency
        self.phase = (self.phase * 12345.6789) % 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saw_range() {
        let mut osc = AnalogOscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let sample = osc.process();
            assert!(sample >= -1.5 && sample <= 1.5, "Saw out of range: {}", sample);
        }
    }

    #[test]
    fn test_sine_range() {
        let mut osc = AnalogOscillator::new(44100.0);
        osc.set_waveform(Waveform::Sine);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let sample = osc.process();
            assert!(sample >= -1.1 && sample <= 1.1, "Sine out of range: {}", sample);
        }
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = AnalogOscillator::new(44100.0);
        osc.set_frequency(220.0);
        assert!((osc.frequency - 220.0).abs() < 0.1);

        osc.set_frequency(880.0);
        assert!((osc.frequency - 880.0).abs() < 0.1);
    }
}
