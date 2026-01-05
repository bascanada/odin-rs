//! Synth Engine - polyphonic voice management

use crate::voice::Voice;
use crate::mod_matrix::ModMatrix;
use crate::constants::*;

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
