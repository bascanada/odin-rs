//! Odin 2 Preset Parser
//!
//! Converts parsed preset data to structured OdinPreset

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::types::*;
use super::value_tree::{ValueTree, ValueTreeError};
use std::fs;
use std::path::Path;

/// Complete Odin 2 preset
#[derive(Debug, Clone, Default)]
pub struct OdinPreset {
    pub name: String,
    pub version_minor: i32,
    pub version_patch: i32,

    // Oscillators
    pub osc1: OscillatorParams,
    pub osc2: OscillatorParams,
    pub osc3: OscillatorParams,

    // Filters
    pub filter1: FilterParams,
    pub filter2: FilterParams,
    pub filter3: FilterParams,

    // Envelopes
    pub env1: EnvelopeParams, // Amp envelope
    pub env2: EnvelopeParams, // Filter envelope
    pub env3: EnvelopeParams, // Mod envelope
    pub env4: EnvelopeParams, // Global envelope

    // LFOs
    pub lfo1: LfoParams,
    pub lfo2: LfoParams,
    pub lfo3: LfoParams,
    pub lfo4: LfoParams,

    // Effects
    pub delay: DelayParams,
    pub reverb: ReverbParams,
    pub chorus: ChorusParams,
    pub phaser: PhaserParams,
    pub flanger: FlangerParams,
    pub distortion: DistortionParams,

    // Modulation matrix (9 rows)
    pub mod_matrix: [ModMatrixRow; 9],

    // Arpeggiator
    pub arpeggiator: ArpeggiatorParams,

    // Global
    pub master: f32,
    pub glide: f32,
    pub unison_detune: f32,
    pub unison_width: f32,
    pub unison_voices: i32,
    pub play_mode: PlayMode,
    pub pitchbend_amount: i32,

    // Amp section
    pub amp_velocity: f32,
    pub amp_pan: f32,
    pub amp_gain: f32,

    // XY Pad
    pub xy_x: f32,
    pub xy_y: f32,
}

impl OdinPreset {
    /// Load preset from file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ValueTreeError> {
        let data = fs::read(path).map_err(ValueTreeError::Io)?;
        Self::from_bytes(&data)
    }

    /// Perform a 2D scatter morph between multiple presets
    ///
    /// # Arguments
    /// * `sources` - Valid presets with their (x, y) coordinates in range [-1.0, 1.0]
    /// * `x` - Target x coordinate [-1.0, 1.0]
    /// * `y` - Target y coordinate [-1.0, 1.0]
    ///
    /// Uses inverse distance weighting to interpolate between an arbitrary number of presets.
    pub fn morph_2d(sources: &[(OdinPreset, f32, f32)], x: f32, y: f32) -> Self {
        if sources.is_empty() {
            return Self::default();
        }

        // Just one preset? Return it directly
        if sources.len() == 1 {
            return sources[0].0.clone();
        }

        // Check for exact matches (avoid division by zero)
        for (preset, px, py) in sources {
            let dx = x - px;
            let dy = y - py;
            if dx * dx + dy * dy < 0.00001 {
                return preset.clone();
            }
        }

        // Calculate weights using Inverse Distance Weighting (IDW)
        // weight = 1 / distance^p (using p=2 for smooth transitions)
        let mut weights = Vec::with_capacity(sources.len());
        let mut total_weight = 0.0;

        for (_, px, py) in sources {
            let dx = x - px;
            let dy = y - py;
            let dist_sq = dx * dx + dy * dy;
            let weight = 1.0 / dist_sq.max(0.00001); // Avoid zero division just in case
            weights.push(weight);
            total_weight += weight;
        }

        // Normalize weights
        for w in &mut weights {
            *w /= total_weight;
        }

        // Mix presets
        let preset_refs: Vec<&OdinPreset> = sources.iter().map(|(p, _, _)| p).collect();
        Self::mix_presets(&preset_refs, &weights)
    }

    /// Parse preset from binary data
    pub fn from_bytes(data: &[u8]) -> Result<Self, ValueTreeError> {
        let tree = ValueTree::from_bytes(data)?;
        Self::from_value_tree(&tree)
    }

    /// Convert ValueTree to OdinPreset
    pub fn from_value_tree(tree: &ValueTree) -> Result<Self, ValueTreeError> {
        let mut preset = OdinPreset::default();

        // Parse misc section properties
        if let Some(name) = tree.get_misc("patch_name") {
            if let Some(s) = name.as_string() {
                preset.name = s.to_string();
            }
        }
        if let Some(v) = tree.get_misc("version_minor") {
            preset.version_minor = v.as_i32().unwrap_or(0);
        }
        if let Some(v) = tree.get_misc("version_patch") {
            preset.version_patch = v.as_i32().unwrap_or(0);
        }
        if let Some(v) = tree.get_misc("legato") {
            preset.play_mode = PlayMode::from(v.as_i32().unwrap_or(1));
        }
        if let Some(v) = tree.get_misc("pitchbend_amount") {
            preset.pitchbend_amount = v.as_i32().unwrap_or(12);
        }
        if let Some(v) = tree.get_misc("unison_voices") {
            preset.unison_voices = v.as_i32().unwrap_or(1);
        }
        if let Some(v) = tree.get_misc("dist_on") {
            preset.distortion.on = v.as_bool().unwrap_or(false);
        }
        if let Some(v) = tree.get_misc("dist_algo") {
            preset.distortion.algorithm = DistortionAlgo::from(v.as_i32().unwrap_or(1));
        }

        // Filter types from misc section
        if let Some(v) = tree.get_misc("fil1_type") {
            preset.filter1.filter_type = FilterType::from(v.as_i32().unwrap_or(0));
        }
        if let Some(v) = tree.get_misc("fil2_type") {
            preset.filter2.filter_type = FilterType::from(v.as_i32().unwrap_or(1));
        }
        if let Some(v) = tree.get_misc("fil3_type") {
            preset.filter3.filter_type = FilterType::from(v.as_i32().unwrap_or(1));
        }

        // Filter comb polarity
        preset.filter1.comb_polarity = get_misc_bool(tree, "fil1_comb_polarity");
        preset.filter2.comb_polarity = get_misc_bool(tree, "fil2_comb_polarity");
        preset.filter3.comb_polarity = get_misc_bool(tree, "fil3_comb_polarity");

        // Filter vowels
        preset.filter1.vowel_left = get_misc_i32(tree, "fil1_vowel_left", 0);
        preset.filter1.vowel_right = get_misc_i32(tree, "fil1_vowel_right", 2);
        preset.filter2.vowel_left = get_misc_i32(tree, "fil2_vowel_left", 0);
        preset.filter2.vowel_right = get_misc_i32(tree, "fil2_vowel_right", 2);
        preset.filter3.vowel_left = get_misc_i32(tree, "fil3_vowel_left", 0);
        preset.filter3.vowel_right = get_misc_i32(tree, "fil3_vowel_right", 2);

        // Arpeggiator settings from misc
        if let Some(v) = tree.get_misc("arp_direction") {
            preset.arpeggiator.direction = ArpDirection::from(v.as_i32().unwrap_or(0));
        }
        preset.arpeggiator.octaves = get_misc_i32(tree, "arp_octaves", 2);
        preset.arpeggiator.steps = get_misc_i32(tree, "arp_steps", 16);
        preset.arpeggiator.gate = get_misc_i32(tree, "arp_gate", 50);
        preset.arpeggiator.sync_numerator = get_misc_i32(tree, "arp_synctime_numerator", 1);
        preset.arpeggiator.sync_denominator = get_misc_i32(tree, "arp_synctime_denominator", 5);

        // Oscillator section properties
        preset.osc1.osc_type = OscillatorType::from(get_osc_i32(tree, "osc1_type", 0));
        preset.osc2.osc_type = OscillatorType::from(get_osc_i32(tree, "osc2_type", 1));
        preset.osc3.osc_type = OscillatorType::from(get_osc_i32(tree, "osc3_type", 1));

        preset.osc1.analog_wave = AnalogWaveform::from(get_osc_i32(tree, "osc1_analog_wave", 0));
        preset.osc2.analog_wave = AnalogWaveform::from(get_osc_i32(tree, "osc2_analog_wave", 0));
        preset.osc3.analog_wave = AnalogWaveform::from(get_osc_i32(tree, "osc3_analog_wave", 0));

        preset.osc1.wavetable = get_osc_i32(tree, "osc1_wavetable", 1);
        preset.osc2.wavetable = get_osc_i32(tree, "osc2_wavetable", 1);
        preset.osc3.wavetable = get_osc_i32(tree, "osc3_wavetable", 1);

        preset.osc1.chipwave = get_osc_i32(tree, "osc1_chipwave", 1);
        preset.osc2.chipwave = get_osc_i32(tree, "osc2_chipwave", 1);
        preset.osc3.chipwave = get_osc_i32(tree, "osc3_chipwave", 1);

        // Vector oscillator settings
        preset.osc1.vec_a = get_osc_i32(tree, "osc1_vec_a", 101);
        preset.osc1.vec_b = get_osc_i32(tree, "osc1_vec_b", 102);
        preset.osc1.vec_c = get_osc_i32(tree, "osc1_vec_c", 103);
        preset.osc1.vec_d = get_osc_i32(tree, "osc1_vec_d", 104);
        preset.osc2.vec_a = get_osc_i32(tree, "osc2_vec_a", 101);
        preset.osc2.vec_b = get_osc_i32(tree, "osc2_vec_b", 102);
        preset.osc2.vec_c = get_osc_i32(tree, "osc2_vec_c", 103);
        preset.osc2.vec_d = get_osc_i32(tree, "osc2_vec_d", 104);
        preset.osc3.vec_a = get_osc_i32(tree, "osc3_vec_a", 101);
        preset.osc3.vec_b = get_osc_i32(tree, "osc3_vec_b", 102);
        preset.osc3.vec_c = get_osc_i32(tree, "osc3_vec_c", 103);
        preset.osc3.vec_d = get_osc_i32(tree, "osc3_vec_d", 104);

        // FM/PM oscillator settings
        preset.osc1.modulator_wave = get_osc_i32(tree, "osc1_modulator_wave", 1);
        preset.osc1.carrier_wave = get_osc_i32(tree, "osc1_carrier_wave", 1);
        preset.osc1.mod_source = get_osc_i32(tree, "osc1_mod_source", 1);
        preset.osc1.carrier_ratio = get_osc_i32(tree, "osc1_carrier_ratio", 1);
        preset.osc1.modulator_ratio = get_osc_i32(tree, "osc1_modulator_ratio", 1);
        preset.osc2.modulator_wave = get_osc_i32(tree, "osc2_modulator_wave", 1);
        preset.osc2.carrier_wave = get_osc_i32(tree, "osc2_carrier_wave", 1);
        preset.osc2.mod_source = get_osc_i32(tree, "osc2_mod_source", 1);
        preset.osc2.carrier_ratio = get_osc_i32(tree, "osc2_carrier_ratio", 1);
        preset.osc2.modulator_ratio = get_osc_i32(tree, "osc2_modulator_ratio", 1);
        preset.osc3.modulator_wave = get_osc_i32(tree, "osc3_modulator_wave", 1);
        preset.osc3.carrier_wave = get_osc_i32(tree, "osc3_carrier_wave", 1);
        preset.osc3.mod_source = get_osc_i32(tree, "osc3_mod_source", 1);
        preset.osc3.carrier_ratio = get_osc_i32(tree, "osc3_carrier_ratio", 1);
        preset.osc3.modulator_ratio = get_osc_i32(tree, "osc3_modulator_ratio", 1);

        // LFO section properties
        preset.lfo1.waveform = LfoWaveform::from(get_lfo_i32(tree, "lfo1_wave", 0));
        preset.lfo2.waveform = LfoWaveform::from(get_lfo_i32(tree, "lfo2_wave", 0));
        preset.lfo3.waveform = LfoWaveform::from(get_lfo_i32(tree, "lfo3_wave", 0));
        preset.lfo4.waveform = LfoWaveform::from(get_lfo_i32(tree, "lfo4_wave", 0));

        preset.lfo1.sync = get_lfo_bool(tree, "lfo1_sync");
        preset.lfo2.sync = get_lfo_bool(tree, "lfo2_sync");
        preset.lfo3.sync = get_lfo_bool(tree, "lfo3_sync");
        preset.lfo4.sync = get_lfo_bool(tree, "lfo4_sync");

        preset.lfo1.sync_numerator = get_lfo_i32(tree, "lfo1_synctime_numerator", 2);
        preset.lfo1.sync_denominator = get_lfo_i32(tree, "lfo1_synctime_denominator", 5);
        preset.lfo2.sync_numerator = get_lfo_i32(tree, "lfo2_synctime_numerator", 2);
        preset.lfo2.sync_denominator = get_lfo_i32(tree, "lfo2_synctime_denominator", 5);
        preset.lfo3.sync_numerator = get_lfo_i32(tree, "lfo3_synctime_numerator", 2);
        preset.lfo3.sync_denominator = get_lfo_i32(tree, "lfo3_synctime_denominator", 5);
        preset.lfo4.sync_numerator = get_lfo_i32(tree, "lfo4_synctime_numerator", 2);
        preset.lfo4.sync_denominator = get_lfo_i32(tree, "lfo4_synctime_denominator", 5);

        // FX section properties
        preset.delay.sync = get_fx_bool(tree, "delay_sync");
        preset.chorus.sync = get_fx_bool(tree, "chorus_sync");
        preset.phaser.sync = get_fx_bool(tree, "phaser_sync");
        preset.flanger.sync = get_fx_bool(tree, "flanger_sync");

        // Mod matrix section properties
        for row in 0..9 {
            preset.mod_matrix[row].source = get_mod_i32(tree, &format!("source_row_{}", row), 0);
            preset.mod_matrix[row].dest1 = get_mod_i32(tree, &format!("dest_1_row_{}", row), 0);
            preset.mod_matrix[row].dest2 = get_mod_i32(tree, &format!("dest_2_row_{}", row), 0);
            preset.mod_matrix[row].scale = get_mod_i32(tree, &format!("scale_row_{}", row), 0);
            preset.mod_matrix[row].amount0 = get_mod_f32(tree, &format!("amount_0_row_{}", row), 0.0);
            preset.mod_matrix[row].amount1 = get_mod_f32(tree, &format!("amount_1_row_{}", row), 0.0);
            preset.mod_matrix[row].amount2 = get_mod_f32(tree, &format!("amount_2_row_{}", row), 0.0);
        }

        // Parse PARAM entries (audio parameters)
        for (name, &value) in &tree.params {
            preset.set_audio_param(name, value as f32);
        }

        Ok(preset)
    }

    /// Set an audio parameter by name
    fn set_audio_param(&mut self, name: &str, value: f32) {
        match name {
            // Oscillator 1
            "osc1_oct" => self.osc1.octave = value as i32,
            "osc1_semi" => self.osc1.semitones = value as i32,
            "osc1_fine" => self.osc1.fine = value,
            "osc1_vol" => self.osc1.volume = value,
            "osc1_reset" => self.osc1.reset = value != 0.0,
            "osc1_position" => self.osc1.position = value,
            "osc1_detune" => self.osc1.detune = value,
            "osc1_pos_mod" => self.osc1.pos_mod = value,
            "osc1_multi_position" => self.osc1.multi_position = value,
            "osc1_spread" => self.osc1.spread = value,
            "osc1_pulsewidth" => self.osc1.pulsewidth = value,
            "osc1_drift" => self.osc1.drift = value,
            "osc1_vec_x" => self.osc1.vec_x = value,
            "osc1_vec_y" => self.osc1.vec_y = value,
            "osc1_arp_on" => self.osc1.arp_on = value != 0.0,
            "osc1_arp_speed" => self.osc1.arp_speed = value,
            "osc1_step_1" => self.osc1.step_1 = value as i32,
            "osc1_step_2" => self.osc1.step_2 = value as i32,
            "osc1_step_3" => self.osc1.step_3 = value as i32,
            "osc1_step_3_on" => self.osc1.step_3_on = value != 0.0,
            "osc1_chipnoise" => self.osc1.chipnoise = value != 0.0,
            "osc1_exp_fm" => self.osc1.exp_fm = value != 0.0,
            "osc1_fm" => self.osc1.fm = value,
            "osc1_lp" => self.osc1.lp = value,
            "osc1_hp" => self.osc1.hp = value,

            // Oscillator 2
            "osc2_oct" => self.osc2.octave = value as i32,
            "osc2_semi" => self.osc2.semitones = value as i32,
            "osc2_fine" => self.osc2.fine = value,
            "osc2_vol" => self.osc2.volume = value,
            "osc2_reset" => self.osc2.reset = value != 0.0,
            "osc2_sync" => self.osc2.sync = value != 0.0,
            "osc2_position" => self.osc2.position = value,
            "osc2_detune" => self.osc2.detune = value,
            "osc2_pos_mod" => self.osc2.pos_mod = value,
            "osc2_multi_position" => self.osc2.multi_position = value,
            "osc2_spread" => self.osc2.spread = value,
            "osc2_pulsewidth" => self.osc2.pulsewidth = value,
            "osc2_drift" => self.osc2.drift = value,
            "osc2_vec_x" => self.osc2.vec_x = value,
            "osc2_vec_y" => self.osc2.vec_y = value,
            "osc2_arp_on" => self.osc2.arp_on = value != 0.0,
            "osc2_arp_speed" => self.osc2.arp_speed = value,
            "osc2_step_1" => self.osc2.step_1 = value as i32,
            "osc2_step_2" => self.osc2.step_2 = value as i32,
            "osc2_step_3" => self.osc2.step_3 = value as i32,
            "osc2_step_3_on" => self.osc2.step_3_on = value != 0.0,
            "osc2_chipnoise" => self.osc2.chipnoise = value != 0.0,
            "osc2_exp_fm" => self.osc2.exp_fm = value != 0.0,
            "osc2_fm" => self.osc2.fm = value,
            "osc2_lp" => self.osc2.lp = value,
            "osc2_hp" => self.osc2.hp = value,

            // Oscillator 3
            "osc3_oct" => self.osc3.octave = value as i32,
            "osc3_semi" => self.osc3.semitones = value as i32,
            "osc3_fine" => self.osc3.fine = value,
            "osc3_vol" => self.osc3.volume = value,
            "osc3_reset" => self.osc3.reset = value != 0.0,
            "osc3_sync" => self.osc3.sync = value != 0.0,
            "osc3_position" => self.osc3.position = value,
            "osc3_detune" => self.osc3.detune = value,
            "osc3_pos_mod" => self.osc3.pos_mod = value,
            "osc3_multi_position" => self.osc3.multi_position = value,
            "osc3_spread" => self.osc3.spread = value,
            "osc3_pulsewidth" => self.osc3.pulsewidth = value,
            "osc3_drift" => self.osc3.drift = value,
            "osc3_vec_x" => self.osc3.vec_x = value,
            "osc3_vec_y" => self.osc3.vec_y = value,
            "osc3_arp_on" => self.osc3.arp_on = value != 0.0,
            "osc3_arp_speed" => self.osc3.arp_speed = value,
            "osc3_step_1" => self.osc3.step_1 = value as i32,
            "osc3_step_2" => self.osc3.step_2 = value as i32,
            "osc3_step_3" => self.osc3.step_3 = value as i32,
            "osc3_step_3_on" => self.osc3.step_3_on = value != 0.0,
            "osc3_chipnoise" => self.osc3.chipnoise = value != 0.0,
            "osc3_exp_fm" => self.osc3.exp_fm = value != 0.0,
            "osc3_fm" => self.osc3.fm = value,
            "osc3_lp" => self.osc3.lp = value,
            "osc3_hp" => self.osc3.hp = value,

            // Filter 1
            "fil1_freq" => self.filter1.frequency = value,
            "fil1_res" => self.filter1.resonance = value,
            "fil1_saturation" => self.filter1.saturation = value,
            "fil1_gain" => self.filter1.gain = value,
            "fil1_env" => self.filter1.env_amount = value,
            "fil1_vel" => self.filter1.vel_amount = value,
            "fil1_kbd" => self.filter1.kbd_follow = value,
            "fil1_sem_transition" => self.filter1.sem_transition = value,
            "fil1_formant_transition" => self.filter1.formant_transition = value,
            "fil1_ring_mod_amount" => self.filter1.ring_mod_amount = value,
            "fil1_osc1" => self.filter1.osc1_input = value != 0.0,
            "fil1_osc2" => self.filter1.osc2_input = value != 0.0,
            "fil1_osc3" => self.filter1.osc3_input = value != 0.0,
            "fil1_to_amp" => self.filter1.to_amp = value != 0.0,

            // Filter 2
            "fil2_freq" => self.filter2.frequency = value,
            "fil2_res" => self.filter2.resonance = value,
            "fil2_saturation" => self.filter2.saturation = value,
            "fil2_gain" => self.filter2.gain = value,
            "fil2_env" => self.filter2.env_amount = value,
            "fil2_vel" => self.filter2.vel_amount = value,
            "fil2_kbd" => self.filter2.kbd_follow = value,
            "fil2_sem_transition" => self.filter2.sem_transition = value,
            "fil2_formant_transition" => self.filter2.formant_transition = value,
            "fil2_ring_mod_amount" => self.filter2.ring_mod_amount = value,
            "fil2_osc1" => self.filter2.osc1_input = value != 0.0,
            "fil2_osc2" => self.filter2.osc2_input = value != 0.0,
            "fil2_osc3" => self.filter2.osc3_input = value != 0.0,
            "fil2_fil1" => self.filter2.fil1_input = value != 0.0,
            "fil2_to_amp" => self.filter2.to_amp = value != 0.0,

            // Filter 3
            "fil3_freq" => self.filter3.frequency = value,
            "fil3_res" => self.filter3.resonance = value,
            "fil3_saturation" => self.filter3.saturation = value,
            "fil3_gain" => self.filter3.gain = value,
            "fil3_env" => self.filter3.env_amount = value,
            "fil3_vel" => self.filter3.vel_amount = value,
            "fil3_kbd" => self.filter3.kbd_follow = value,
            "fil3_sem_transition" => self.filter3.sem_transition = value,
            "fil3_formant_transition" => self.filter3.formant_transition = value,
            "fil3_ring_mod_amount" => self.filter3.ring_mod_amount = value,

            // Envelopes
            "env1_attack" => self.env1.attack = value,
            "env1_decay" => self.env1.decay = value,
            "env1_sustain" => self.env1.sustain = value,
            "env1_release" => self.env1.release = value,
            "env1_loop" => self.env1.loop_env = value != 0.0,

            "env2_attack" => self.env2.attack = value,
            "env2_decay" => self.env2.decay = value,
            "env2_sustain" => self.env2.sustain = value,
            "env2_release" => self.env2.release = value,
            "env2_loop" => self.env2.loop_env = value != 0.0,

            "env3_attack" => self.env3.attack = value,
            "env3_decay" => self.env3.decay = value,
            "env3_sustain" => self.env3.sustain = value,
            "env3_release" => self.env3.release = value,
            "env3_loop" => self.env3.loop_env = value != 0.0,

            "env4_attack" => self.env4.attack = value,
            "env4_decay" => self.env4.decay = value,
            "env4_sustain" => self.env4.sustain = value,
            "env4_release" => self.env4.release = value,
            "env4_loop" => self.env4.loop_env = value != 0.0,

            // LFOs
            "lfo1_freq" => self.lfo1.frequency = value,
            "lfo1_reset" => self.lfo1.reset = value != 0.0,
            "lfo2_freq" => self.lfo2.frequency = value,
            "lfo2_reset" => self.lfo2.reset = value != 0.0,
            "lfo3_freq" => self.lfo3.frequency = value,
            "lfo3_reset" => self.lfo3.reset = value != 0.0,
            "lfo4_freq" => self.lfo4.frequency = value,
            "lfo4_reset" => self.lfo4.reset = value != 0.0,

            // Delay
            "delay_on" => self.delay.on = value != 0.0,
            "delay_time" => self.delay.time = value,
            "delay_feedback" => self.delay.feedback = value,
            "delay_dry" => self.delay.dry = value,
            "delay_wet" => self.delay.wet = value,
            "delay_hp" => self.delay.hp = value,
            "delay_ducking" => self.delay.ducking = value,
            "delay_pingpong" => self.delay.pingpong = value != 0.0,

            // Reverb
            "reverb_on" => self.reverb.on = value != 0.0,
            "rev_delay" => self.reverb.delay = value,
            "rev_drywet" => self.reverb.dry_wet = value,
            "rev_mid_hall" => self.reverb.mid_hall = value,
            "rev_hf_damp" => self.reverb.hf_damp = value,
            "rev_eqgain" => self.reverb.eq_gain = value,
            "rev_eqfreq" => self.reverb.eq_freq = value,

            // Chorus
            "chorus_on" => self.chorus.on = value != 0.0,
            "chorus_amount" => self.chorus.amount = value,
            "chorus_rate" => self.chorus.rate = value,
            "chorus_feedback" => self.chorus.feedback = value,
            "chorus_drywet" => self.chorus.dry_wet = value,
            "chorus_reset" => self.chorus.reset = value != 0.0,

            // Phaser
            "phaser_on" => self.phaser.on = value != 0.0,
            "phaser_freq" => self.phaser.frequency = value,
            "phaser_feedback" => self.phaser.feedback = value,
            "phaser_rate" => self.phaser.rate = value,
            "phaser_mod" => self.phaser.mod_amount = value,
            "phaser_drywet" => self.phaser.dry_wet = value,
            "phaser_reset" => self.phaser.reset = value != 0.0,

            // Flanger
            "flanger_on" => self.flanger.on = value != 0.0,
            "flanger_amount" => self.flanger.amount = value,
            "flanger_rate" => self.flanger.rate = value,
            "flanger_feedback" => self.flanger.feedback = value,
            "flanger_drywet" => self.flanger.dry_wet = value,
            "flanger_reset" => self.flanger.reset = value != 0.0,

            // Distortion
            "dist_boost" => self.distortion.boost = value,
            "dist_drywet" => self.distortion.dry_wet = value,

            // Amp
            "amp_velocity" => self.amp_velocity = value,
            "amp_pan" => self.amp_pan = value,
            "amp_gain" => self.amp_gain = value,

            // Global
            "master" => self.master = value,
            "glide" => self.glide = value,
            "unison_detune" => self.unison_detune = value,
            "unison_width" => self.unison_width = value,
            "xy_x" => self.xy_x = value,
            "xy_y" => self.xy_y = value,

            // Arpeggiator
            "arp_on" => self.arpeggiator.on = value != 0.0,
            "arp_one_shot" => self.arpeggiator.one_shot = value != 0.0,
            "arp_mod_transpose" => self.arpeggiator.mod_transpose = value,

            // Arp steps
            name if name.starts_with("step_") => {
                self.set_arp_step_param(name, value);
            }

            _ => {}
        }
    }

    fn set_arp_step_param(&mut self, name: &str, value: f32) {
        // Parse step_N_PARAM format
        let parts: Vec<&str> = name.split('_').collect();
        if parts.len() >= 3 {
            if let Ok(step_num) = parts[1].parse::<usize>() {
                if step_num < 16 {
                    match parts[2] {
                        "on" => self.arpeggiator.step_params[step_num].on = value != 0.0,
                        "transpose" => {
                            self.arpeggiator.step_params[step_num].transpose = value as i32
                        }
                        "mod" if parts.len() >= 4 => match parts[3] {
                            "1" => self.arpeggiator.step_params[step_num].mod1 = value,
                            "2" => self.arpeggiator.step_params[step_num].mod2 = value,
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

// Helper functions for section property access
fn get_misc_bool(tree: &ValueTree, name: &str) -> bool {
    tree.get_misc(name).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn get_misc_i32(tree: &ValueTree, name: &str, default: i32) -> i32 {
    tree.get_misc(name).and_then(|v| v.as_i32()).unwrap_or(default)
}

fn get_osc_i32(tree: &ValueTree, name: &str, default: i32) -> i32 {
    tree.get_osc(name).and_then(|v| v.as_i32()).unwrap_or(default)
}

fn get_lfo_bool(tree: &ValueTree, name: &str) -> bool {
    tree.get_lfo(name).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn get_lfo_i32(tree: &ValueTree, name: &str, default: i32) -> i32 {
    tree.get_lfo(name).and_then(|v| v.as_i32()).unwrap_or(default)
}

fn get_fx_bool(tree: &ValueTree, name: &str) -> bool {
    tree.get_fx(name).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn get_mod_i32(tree: &ValueTree, name: &str, default: i32) -> i32 {
    tree.get_mod(name).and_then(|v| v.as_i32()).unwrap_or(default)
}

fn get_mod_f32(tree: &ValueTree, name: &str, default: f32) -> f32 {
    tree.get_mod(name).and_then(|v| v.as_f32()).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::println;
    use std::vec;

    #[test]
    fn test_load_preset() {
        // Test with a real preset file if available
        let path = "/Users/william.quintal/Project/bascanada/odin2-rs/odin2/assets/Soundbanks/Factory Presets/Bass/Analog Bass [tx].odin";
        if std::path::Path::new(path).exists() {
            let result = OdinPreset::load(path);
            match result {
                Ok(preset) => {
                    println!("Loaded preset: {}", preset.name);
                    println!("Osc1 type: {:?}", preset.osc1.osc_type);
                    println!("Osc1 volume: {}", preset.osc1.volume);
                    println!("Filter1 type: {:?}", preset.filter1.filter_type);
                    println!("Filter1 freq: {}", preset.filter1.frequency);
                    println!("Env1 attack: {}", preset.env1.attack);
                    println!("Env1 decay: {}", preset.env1.decay);
                    println!("Env1 sustain: {}", preset.env1.sustain);
                    println!("Env1 release: {}", preset.env1.release);
                }
                Err(e) => {
                    println!("Error loading preset: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_2d_morphing() {
        // Create 4 test presets
        let mut p1 = OdinPreset::default();
        p1.name = "P1".to_string();
        p1.filter1.frequency = 100.0; // Top-Left (-1, 1)

        let mut p2 = OdinPreset::default();
        p2.name = "P2".to_string();
        p2.filter1.frequency = 200.0; // Top-Right (1, 1)

        let mut p3 = OdinPreset::default();
        p3.name = "P3".to_string();
        p3.filter1.frequency = 300.0; // Bottom-Left (-1, -1)

        let mut p4 = OdinPreset::default();
        p4.name = "P4".to_string();
        p4.filter1.frequency = 400.0; // Bottom-Right (1, -1)

        let sources = vec![
            (p1, -1.0, 1.0),
            (p2, 1.0, 1.0),
            (p3, -1.0, -1.0),
            (p4, 1.0, -1.0),
        ];

        // Test exact corners
        let morph_p1 = OdinPreset::morph_2d(&sources, -1.0, 1.0);
        assert!((morph_p1.filter1.frequency - 100.0).abs() < 0.001);

        let morph_p4 = OdinPreset::morph_2d(&sources, 1.0, -1.0);
        assert!((morph_p4.filter1.frequency - 400.0).abs() < 0.001);

        // Test center (should be average if symmetric)
        // At (0,0), dist to all corners is sqrt(2). Weights are equal.
        // Average: (100+200+300+400)/4 = 250
        let morph_center = OdinPreset::morph_2d(&sources, 0.0, 0.0);
        assert!((morph_center.filter1.frequency - 250.0).abs() < 1.0);

        println!("✓ 2D scatter morphing works correctly");
    }

    #[test]
    fn test_load_all_factory_presets() {
        use std::fs;

        let base_path = "/Users/william.quintal/Project/bascanada/odin2-rs/odin2/assets/Soundbanks/Factory Presets";
        if !std::path::Path::new(base_path).exists() {
            return;
        }

        let mut success_count = 0;
        let mut fail_count = 0;
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        // Walk through all subdirectories
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let category = entry.file_name().to_string_lossy().to_string();
                    if let Ok(files) = fs::read_dir(entry.path()) {
                        for file in files.flatten() {
                            if file.path().extension().map_or(false, |ext| ext == "odin") {
                                match OdinPreset::load(file.path()) {
                                    Ok(_) => {
                                        success_count += 1;
                                        *categories.entry(category.clone()).or_insert(0) += 1;
                                    }
                                    Err(e) => {
                                        fail_count += 1;
                                        println!("FAIL: {:?} - {}", file.path().file_name(), e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("\n=== Factory Preset Test Results ===");
        println!("Success: {} presets", success_count);
        println!("Failed:  {} presets", fail_count);
        println!("\nBy category:");
        for (category, count) in &categories {
            println!("  {}: {} presets", category, count);
        }

        // All factory presets should load successfully
        assert_eq!(fail_count, 0, "Some presets failed to load");
    }
}
