//! Voice module - single synthesizer voice

use crate::dsp::oscillators::{AnalogOscillator, Oscillator};
use crate::dsp::filters::{LadderFilter, Filter};
use crate::dsp::envelopes::{Adsr, Envelope};
use crate::constants::*;

/// A single synthesizer voice
pub struct Voice {
    /// Oscillators (3 per voice)
    pub oscillators: [AnalogOscillator; OSCILLATORS_PER_VOICE],

    /// Filter 1
    pub filter1: LadderFilter,

    /// Filter 2
    pub filter2: LadderFilter,

    /// Amplitude envelope
    pub amp_env: Adsr,

    /// Filter envelope
    pub filter_env: Adsr,

    /// Modulation envelope
    pub mod_env: Adsr,

    /// Currently playing MIDI note
    pub note: u8,

    /// Velocity (0.0 to 1.0)
    pub velocity: f32,

    /// Whether voice is active
    pub active: bool,

    /// Sample rate
    sample_rate: f32,

    /// Oscillator volumes from preset (default: [1.0, 1.0, 1.0])
    pub osc_volumes: [f32; 3],

    /// Filter envelope amount from preset (default: 0.0)
    pub filter_env_amount: f32,

    /// Base filter cutoff frequency (before envelope modulation)
    pub filter_base_cutoff: f32,
}

impl Voice {
    /// Create a new voice
    pub fn new(sample_rate: f32) -> Self {
        Self {
            oscillators: [
                AnalogOscillator::new(sample_rate),
                AnalogOscillator::new(sample_rate),
                AnalogOscillator::new(sample_rate),
            ],
            filter1: LadderFilter::new(sample_rate),
            filter2: LadderFilter::new(sample_rate),
            amp_env: Adsr::new(sample_rate),
            filter_env: Adsr::new(sample_rate),
            mod_env: Adsr::new(sample_rate),
            note: 0,
            velocity: 0.0,
            active: false,
            sample_rate,
            osc_volumes: [1.0, 1.0, 1.0],    // Default: all oscillators at full volume
            filter_env_amount: 0.0,          // Default: no filter envelope modulation
            filter_base_cutoff: 1000.0,      // Default: 1kHz
        }
    }

    /// Start a note
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        self.note = note;
        self.velocity = velocity as f32 / 127.0;
        self.active = true;

        // Set oscillator frequencies
        let freq = crate::dsp::midi_to_freq(note);
        for osc in &mut self.oscillators {
            osc.set_frequency(freq);
        }

        // Trigger envelopes
        self.amp_env.trigger();
        self.filter_env.trigger();
        self.mod_env.trigger();
    }

    /// Stop a note
    pub fn note_off(&mut self) {
        self.amp_env.release();
        self.filter_env.release();
        self.mod_env.release();
    }

    /// Process one sample (stereo output)
    pub fn process(&mut self) -> (f32, f32) {
        if !self.active {
            return (0.0, 0.0);
        }

        // Process envelopes
        let amp = self.amp_env.process();
        let filter_mod = self.filter_env.process();
        let _mod = self.mod_env.process();

        // Check if voice should be deactivated
        if !self.amp_env.is_active() {
            self.active = false;
            return (0.0, 0.0);
        }

        // Mix oscillators WITH preset volumes
        let mut osc_out = 0.0;
        for (i, osc) in self.oscillators.iter_mut().enumerate() {
            osc_out += osc.process() * self.osc_volumes[i];
        }
        osc_out /= OSCILLATORS_PER_VOICE as f32;

        // Apply filter WITH envelope modulation
        // Modulate filter cutoff by envelope amount
        let mod_cutoff = self.filter_base_cutoff + filter_mod * self.filter_env_amount * FILTER_ENV_MODULATION_HZ;
        self.filter1.set_cutoff(mod_cutoff.clamp(20.0, 20000.0));

        let filtered = self.filter1.process(osc_out);

        // Apply amplitude envelope and velocity
        let output = filtered * amp * self.velocity;

        // Mono to stereo (for now)
        (output, output)
    }

    /// Set sample rate for all components
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for osc in &mut self.oscillators {
            osc.set_sample_rate(sample_rate);
        }
        self.filter1.set_sample_rate(sample_rate);
        self.filter2.set_sample_rate(sample_rate);
        self.amp_env.set_sample_rate(sample_rate);
        self.filter_env.set_sample_rate(sample_rate);
        self.mod_env.set_sample_rate(sample_rate);
    }

    /// Reset voice state
    pub fn reset(&mut self) {
        self.active = false;
        self.note = 0;
        self.velocity = 0.0;
        for osc in &mut self.oscillators {
            osc.reset();
        }
        self.filter1.reset();
        self.filter2.reset();
        self.amp_env.reset();
        self.filter_env.reset();
        self.mod_env.reset();
    }
}
