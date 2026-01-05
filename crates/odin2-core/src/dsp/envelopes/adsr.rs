//! ADSR Envelope Generator
//!
//! Based on the Odin 2 ADSREnvelope implementation.
//! Uses linear attack and exponential decay/release.

use super::{Envelope, EnvelopeState};
use libm::powf;

/// Minimum value before envelope is considered finished
const MIN_DECAY_RELEASE_VAL: f32 = 0.001;

/// Minimum attack time to avoid division by zero
const ATTACK_LOWER_LIMIT: f32 = 0.000001;

/// ADSR Envelope Generator
pub struct Adsr {
    /// Current envelope state
    state: EnvelopeState,

    /// Current value (internal, 0.0 to 1.0)
    current_value: f32,

    /// Last output value
    last_output: f32,

    /// Attack time in seconds
    attack: f32,

    /// Decay time in seconds
    decay: f32,

    /// Sustain level (0.0 to 1.0)
    sustain: f32,

    /// Release time in seconds
    release: f32,

    /// Sample rate
    sample_rate: f32,

    /// Cached decay factor
    decay_factor: f32,

    /// Last decay time (for caching)
    last_decay_time: f32,

    /// Cached release factor
    release_factor: f32,

    /// Last release time (for caching)
    last_release_time: f32,

    /// Value when release started
    release_start_value: f32,

    /// Whether envelope loops (for LFO-like behavior)
    looping: bool,
}

impl Adsr {
    /// Create a new ADSR envelope
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state: EnvelopeState::Idle,
            current_value: 0.0,
            last_output: 0.0,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            sample_rate,
            decay_factor: 0.9998,
            last_decay_time: -1.0,
            release_factor: 0.9998,
            last_release_time: -1.0,
            release_start_value: 1.0,
            looping: false,
        }
    }

    /// Set attack time in seconds
    pub fn set_attack(&mut self, attack: f32) {
        self.attack = attack.max(ATTACK_LOWER_LIMIT);
    }

    /// Set decay time in seconds
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.max(ATTACK_LOWER_LIMIT);
    }

    /// Set sustain level (0.0 to 1.0)
    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(0.0, 1.0);
    }

    /// Set release time in seconds
    pub fn set_release(&mut self, release: f32) {
        self.release = release.max(ATTACK_LOWER_LIMIT);
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Invalidate cached factors
        self.last_decay_time = -1.0;
        self.last_release_time = -1.0;
    }

    /// Set looping mode
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Calculate decay/release factor
    ///
    /// Factor is computed so that: factor^(samples) = MIN_DECAY_RELEASE_VAL
    /// This gives an exponential curve that reaches MIN_DECAY_RELEASE_VAL
    /// in the specified time.
    #[inline]
    fn calc_exp_factor(&self, time: f32) -> f32 {
        let samples = self.sample_rate * time;
        powf(MIN_DECAY_RELEASE_VAL, 1.0 / samples)
    }

    /// Get decay factor (with caching)
    fn get_decay_factor(&mut self) -> f32 {
        if self.decay != self.last_decay_time {
            self.decay_factor = self.calc_exp_factor(self.decay);
            self.last_decay_time = self.decay;
        }
        self.decay_factor
    }

    /// Get release factor (with caching)
    fn get_release_factor(&mut self) -> f32 {
        if self.release != self.last_release_time {
            self.release_factor = self.calc_exp_factor(self.release);
            self.last_release_time = self.release;
        }
        self.release_factor
    }
}

impl Envelope for Adsr {
    fn process(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => {
                self.last_output = 0.0;
                0.0
            }

            EnvelopeState::Attack => {
                // Linear attack
                let attack_inc = 1.0 / (self.sample_rate * self.attack);
                self.current_value += attack_inc;

                if self.current_value >= 1.0 {
                    self.current_value = 1.0;
                    self.state = EnvelopeState::Decay;
                }

                self.last_output = self.current_value;
                self.current_value
            }

            EnvelopeState::Decay => {
                // Exponential decay towards sustain
                let factor = self.get_decay_factor();
                self.current_value *= factor;

                if self.current_value < MIN_DECAY_RELEASE_VAL {
                    if self.looping {
                        self.state = EnvelopeState::Attack;
                        self.current_value = self.sustain;
                    } else {
                        self.state = EnvelopeState::Sustain;
                        self.current_value = 0.0;
                    }
                }

                // Scale output: sustain + (1 - sustain) * current_value
                // This maps the exponential decay from 1->0 to 1->sustain
                self.last_output = self.sustain + (1.0 - self.sustain) * self.current_value;
                self.last_output
            }

            EnvelopeState::Sustain => {
                if self.looping {
                    self.state = EnvelopeState::Attack;
                    self.current_value = self.sustain;
                }
                self.last_output = self.sustain;
                self.sustain
            }

            EnvelopeState::Release => {
                // Exponential release towards zero
                let factor = self.get_release_factor();
                self.current_value *= factor;

                if self.current_value < MIN_DECAY_RELEASE_VAL {
                    self.current_value = 0.0;
                    self.state = EnvelopeState::Idle;
                    self.last_output = 0.0;
                    return 0.0;
                }

                // Scale output by the value when release started
                self.last_output = self.current_value * self.release_start_value;
                self.last_output
            }
        }
    }

    fn trigger(&mut self) {
        // Start from current value (for legato)
        self.current_value = self.last_output;
        self.state = EnvelopeState::Attack;
    }

    fn release(&mut self) {
        if self.state == EnvelopeState::Release {
            return;
        }

        // Store the current output value as the release starting point
        match self.state {
            EnvelopeState::Decay => {
                self.release_start_value =
                    self.current_value * (1.0 - self.sustain) + self.sustain;
            }
            EnvelopeState::Sustain => {
                self.release_start_value = self.sustain;
            }
            _ => {
                self.release_start_value = self.last_output;
            }
        }

        self.current_value = 1.0;
        self.state = EnvelopeState::Release;
    }

    fn state(&self) -> EnvelopeState {
        self.state
    }

    fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }

    fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.current_value = 0.0;
        self.last_output = 0.0;
        self.release_start_value = 1.0;
        self.last_decay_time = -1.0;
        self.last_release_time = -1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adsr_trigger_and_release() {
        let mut env = Adsr::new(44100.0);
        env.set_attack(0.01);
        env.set_decay(0.1);
        env.set_sustain(0.5);
        env.set_release(0.1);

        // Initially idle
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert!(!env.is_active());

        // Trigger
        env.trigger();
        assert_eq!(env.state(), EnvelopeState::Attack);
        assert!(env.is_active());

        // Process through attack
        let mut max_val = 0.0;
        for _ in 0..1000 {
            let val = env.process();
            if val > max_val {
                max_val = val;
            }
        }
        assert!(max_val > 0.9, "Attack should reach near 1.0");

        // Process through decay
        for _ in 0..10000 {
            env.process();
        }

        // Should be at sustain
        assert_eq!(env.state(), EnvelopeState::Sustain);
        let val = env.process();
        assert!((val - 0.5).abs() < 0.01, "Should be at sustain level");

        // Release
        env.release();
        assert_eq!(env.state(), EnvelopeState::Release);

        // Process through release
        for _ in 0..10000 {
            env.process();
        }

        // Should be idle
        assert_eq!(env.state(), EnvelopeState::Idle);
        assert!(!env.is_active());
    }

    #[test]
    fn test_adsr_range() {
        let mut env = Adsr::new(44100.0);
        env.set_attack(0.001);
        env.set_decay(0.05);
        env.set_sustain(0.7);
        env.set_release(0.05);

        env.trigger();

        for _ in 0..50000 {
            let val = env.process();
            assert!(val >= 0.0 && val <= 1.0, "Envelope out of range: {}", val);
        }
    }
}
