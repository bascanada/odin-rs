//! Synth Engine - polyphonic voice management

use crate::voice::Voice;
use crate::mod_matrix::ModMatrix;
use crate::constants::*;
use crate::dsp::oscillators::Oscillator;
use crate::dsp::filters::Filter;

#[cfg(feature = "std")]
use crate::preset::OdinPreset;

/// Generic synthesizer engine trait
pub trait SynthEngine {
    /// Handle note on event
    fn note_on(&mut self, note: u8, velocity: u8);

    /// Handle note off event
    fn note_off(&mut self, note: u8);

    /// Handle control change
    fn control_change(&mut self, cc: u8, value: u8);

    /// Handle pitch bend
    fn pitch_bend(&mut self, value: i16);

    /// Process audio buffer
    fn process(&mut self, output: &mut [f32], channels: usize);

    /// Set sample rate
    fn set_sample_rate(&mut self, sample_rate: f32);
}

/// Voice allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceAllocation {
    /// Round robin allocation
    RoundRobin,
    /// Steal oldest voice
    StealOldest,
    /// Steal quietest voice
    StealQuietest,
}

/// Simplified preset configuration for engine-level preset loading
///
/// This struct extracts essential parameters from an OdinPreset and maps them
/// to the engine's current capabilities (analog oscillators, basic filters, envelopes).
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct PresetConfig {
    // Oscillators (simplified to analog)
    pub osc_volumes: [f32; 3],
    pub osc_octaves: [i32; 3],
    pub osc_semitones: [i32; 3],
    pub osc_detune: f32,

    // Filter
    pub filter_frequency: f32,
    pub filter_resonance: f32,
    pub filter_env_amount: f32,

    // Amp envelope
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,

    // Filter envelope
    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,

    // Master
    pub master_volume: f32,
}

#[cfg(feature = "std")]
impl PresetConfig {
    /// Extract essential parameters from an OdinPreset
    pub fn from_preset(preset: &OdinPreset) -> Self {
        Self {
            // Oscillator parameters
            osc_volumes: [preset.osc1.volume, preset.osc2.volume, preset.osc3.volume],
            osc_octaves: [preset.osc1.octave, preset.osc2.octave, preset.osc3.octave],
            osc_semitones: [
                preset.osc1.semitones,
                preset.osc2.semitones,
                preset.osc3.semitones,
            ],
            osc_detune: preset.osc1.detune,

            // Filter parameters (from filter1)
            filter_frequency: preset.filter1.frequency.clamp(20.0, 20000.0),
            filter_resonance: preset.filter1.resonance,
            filter_env_amount: preset.filter1.env_amount,

            // Amp envelope (env1)
            amp_attack: preset.env1.attack.max(0.001),
            amp_decay: preset.env1.decay.max(0.001),
            amp_sustain: preset.env1.sustain,
            amp_release: preset.env1.release.max(0.01),

            // Filter envelope (env2)
            filter_attack: preset.env2.attack.max(0.001),
            filter_decay: preset.env2.decay.max(0.001),
            filter_sustain: preset.env2.sustain,
            filter_release: preset.env2.release.max(0.01),

            // Master volume
            master_volume: if preset.master > 0.0 {
                preset.master.clamp(0.0, 1.0)
            } else {
                0.7
            },
        }
    }
}

/// The main Odin 2 synthesizer engine
pub struct OdinEngine {
    /// Polyphonic voices
    voices: [Voice; VOICES],

    /// Modulation matrix
    pub mod_matrix: ModMatrix,

    /// Voice allocation strategy
    allocation: VoiceAllocation,

    /// Next voice index for round-robin
    next_voice: usize,

    /// Sample rate
    sample_rate: f32,

    /// Master volume
    pub master_volume: f32,

    /// Currently active notes (for note-off matching)
    note_to_voice: [Option<usize>; 128],

    /// Current preset configuration
    #[cfg(feature = "std")]
    preset_config: Option<PresetConfig>,
}

impl OdinEngine {
    /// Create a new engine
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: core::array::from_fn(|_| Voice::new(sample_rate)),
            mod_matrix: ModMatrix::new(),
            allocation: VoiceAllocation::RoundRobin,
            next_voice: 0,
            sample_rate,
            master_volume: 1.0,
            note_to_voice: [None; 128],
            #[cfg(feature = "std")]
            preset_config: None,
        }
    }

    /// Load a preset into the engine
    ///
    /// This will apply the preset's parameters to all new voices that are triggered.
    /// Currently active voices will not be affected.
    #[cfg(feature = "std")]
    pub fn load_preset(&mut self, preset: &OdinPreset) {
        let config = PresetConfig::from_preset(preset);
        self.master_volume = config.master_volume;
        self.preset_config = Some(config);
    }

    /// Apply preset configuration to a specific voice
    #[cfg(feature = "std")]
    fn apply_preset_to_voice(&mut self, voice_idx: usize) {
        if let Some(ref config) = self.preset_config {
            let voice = &mut self.voices[voice_idx];
            let note = voice.note;

            // Apply oscillator pitch offsets and volumes
            for (i, osc) in voice.oscillators.iter_mut().enumerate() {
                let pitch_offset = (config.osc_octaves[i] * 12 + config.osc_semitones[i]) as f32;
                let freq = crate::dsp::midi_to_freq(note)
                    * libm::powf(2.0, pitch_offset / 12.0);
                osc.set_frequency(freq);
            }

            // Store volumes for processing
            voice.osc_volumes = config.osc_volumes;

            // Apply filter settings
            voice.filter_base_cutoff = config.filter_frequency;
            voice.filter1.set_cutoff(config.filter_frequency);
            voice.filter1.set_resonance(config.filter_resonance);
            voice.filter_env_amount = config.filter_env_amount;

            // Apply envelopes
            voice.amp_env.set_attack(config.amp_attack);
            voice.amp_env.set_decay(config.amp_decay);
            voice.amp_env.set_sustain(config.amp_sustain);
            voice.amp_env.set_release(config.amp_release);

            voice.filter_env.set_attack(config.filter_attack);
            voice.filter_env.set_decay(config.filter_decay);
            voice.filter_env.set_sustain(config.filter_sustain);
            voice.filter_env.set_release(config.filter_release);
        }
    }

    /// Find a free voice or steal one
    fn allocate_voice(&mut self) -> usize {
        // First, try to find an inactive voice
        for (i, voice) in self.voices.iter().enumerate() {
            if !voice.active {
                return i;
            }
        }

        // All voices active, need to steal
        match self.allocation {
            VoiceAllocation::RoundRobin => {
                let voice = self.next_voice;
                self.next_voice = (self.next_voice + 1) % VOICES;
                voice
            }
            VoiceAllocation::StealOldest => {
                // For now, just use round-robin
                let voice = self.next_voice;
                self.next_voice = (self.next_voice + 1) % VOICES;
                voice
            }
            VoiceAllocation::StealQuietest => {
                // Find voice with lowest amplitude envelope value
                let mut quietest = 0;
                let mut min_amp = f32::MAX;
                for (i, voice) in self.voices.iter().enumerate() {
                    let amp = voice.velocity; // Simplified - should use envelope value
                    if amp < min_amp {
                        min_amp = amp;
                        quietest = i;
                    }
                }
                quietest
            }
        }
    }
}

impl SynthEngine for OdinEngine {
    fn note_on(&mut self, note: u8, velocity: u8) {
        // Don't trigger if velocity is 0 (some MIDI devices send note-on with vel 0 for note-off)
        if velocity == 0 {
            self.note_off(note);
            return;
        }

        // Allocate a voice
        let voice_idx = self.allocate_voice();

        // Clear old mapping if this voice was playing another note
        if let Some(old_note) = self.voices[voice_idx].active.then_some(self.voices[voice_idx].note) {
            self.note_to_voice[old_note as usize] = None;
        }

        // Start the note
        self.voices[voice_idx].note_on(note, velocity);

        // Apply preset configuration if available
        #[cfg(feature = "std")]
        self.apply_preset_to_voice(voice_idx);

        self.note_to_voice[note as usize] = Some(voice_idx);
    }

    fn note_off(&mut self, note: u8) {
        if let Some(voice_idx) = self.note_to_voice[note as usize] {
            self.voices[voice_idx].note_off();
            self.note_to_voice[note as usize] = None;
        }
    }

    fn control_change(&mut self, cc: u8, value: u8) {
        match cc {
            1 => {
                // Mod wheel - could route to mod matrix
            }
            7 => {
                // Volume
                self.master_volume = value as f32 / 127.0;
            }
            64 => {
                // Sustain pedal
                // TODO: implement sustain
            }
            _ => {}
        }
    }

    fn pitch_bend(&mut self, _value: i16) {
        // TODO: implement pitch bend
    }

    fn process(&mut self, output: &mut [f32], channels: usize) {
        // Clear output buffer
        output.fill(0.0);

        let samples = output.len() / channels;

        for sample_idx in 0..samples {
            let mut left = 0.0;
            let mut right = 0.0;

            // Sum all voices
            for voice in &mut self.voices {
                if voice.active {
                    let (l, r) = voice.process();
                    left += l;
                    right += r;
                }
            }

            // Apply master volume
            left *= self.master_volume;
            right *= self.master_volume;

            // Write to output buffer
            let base_idx = sample_idx * channels;
            if channels >= 1 {
                output[base_idx] = left;
            }
            if channels >= 2 {
                output[base_idx + 1] = right;
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for voice in &mut self.voices {
            voice.set_sample_rate(sample_rate);
        }
    }
}
