//! Odin 2 Preset Morphing Demo
//!
//! Demonstrates smooth morphing between factory presets.
//! This is a proof-of-concept for real-time preset interpolation in Harmonium.
//!
//! Usage:
//!   cargo run --bin morph-demo --features std -- [OPTIONS]

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use hound::{WavSpec, WavWriter};

use odin2_core::dsp::effects::{Chorus, Delay, ZitaReverb};
use odin2_core::dsp::envelopes::{Adsr, Envelope};
use odin2_core::dsp::filters::{Filter, LadderFilter, LadderFilterType};
use odin2_core::dsp::oscillators::MultiOscillator;
use odin2_core::preset::OdinPreset;

const SAMPLE_RATE: u32 = 44100;

/// Melody definition
struct Melody {
    name: &'static str,
    notes: &'static [(u8, f32, f32)], // (midi_note, start_time, duration)
}

/// Available melodies for morphing
const MELODIES: &[Melody] = &[
    Melody {
        name: "slow_arpeggio",
        notes: &[
            (48, 0.0, 0.8),   // C3
            (52, 1.0, 0.8),   // E3
            (55, 2.0, 0.8),   // G3
            (60, 3.0, 0.8),   // C4
            (64, 4.0, 0.8),   // E4
            (67, 5.0, 0.8),   // G4
            (72, 6.0, 0.8),   // C5
            (67, 7.0, 0.8),   // G4
            (64, 8.0, 0.8),   // E4
            (60, 9.0, 0.8),   // C4
            (55, 10.0, 0.8),  // G3
            (48, 11.0, 1.5),  // C3 (long)
        ],
    },
    Melody {
        name: "chord_stabs",
        notes: &[
            // Chord 1 - C major
            (48, 0.0, 0.6), (60, 0.0, 0.6), (64, 0.0, 0.6), (67, 0.0, 0.6),
            // Chord 2 - F major
            (53, 2.0, 0.6), (60, 2.0, 0.6), (65, 2.0, 0.6), (69, 2.0, 0.6),
            // Chord 3 - Am
            (45, 4.0, 0.6), (57, 4.0, 0.6), (60, 4.0, 0.6), (64, 4.0, 0.6),
            // Chord 4 - G
            (43, 6.0, 0.6), (55, 6.0, 0.6), (59, 6.0, 0.6), (67, 6.0, 0.6),
            // Chord 5 - C major (long)
            (48, 8.0, 2.0), (60, 8.0, 2.0), (64, 8.0, 2.0), (67, 8.0, 2.0),
        ],
    },
    Melody {
        name: "lead_line",
        notes: &[
            (60, 0.0, 0.3),   // C4
            (62, 0.5, 0.3),   // D4
            (64, 1.0, 0.5),   // E4
            (67, 1.75, 0.25), // G4
            (72, 2.0, 0.8),   // C5
            (71, 3.0, 0.3),   // B4
            (69, 3.5, 0.3),   // A4
            (67, 4.0, 0.8),   // G4
            (64, 5.0, 0.3),   // E4
            (62, 5.5, 0.3),   // D4
            (60, 6.0, 0.5),   // C4
            (64, 6.75, 0.25), // E4
            (67, 7.0, 0.5),   // G4
            (72, 7.75, 0.25), // C5
            (76, 8.0, 1.5),   // E5 (long)
        ],
    },
    Melody {
        name: "bass_groove",
        notes: &[
            (36, 0.0, 0.3),   // C2
            (36, 0.5, 0.2),   // C2
            (36, 1.0, 0.3),   // C2
            (38, 1.5, 0.2),   // D2
            (40, 2.0, 0.3),   // E2
            (40, 2.5, 0.2),   // E2
            (43, 3.0, 0.3),   // G2
            (40, 3.5, 0.2),   // E2
            (36, 4.0, 0.3),   // C2
            (36, 4.5, 0.2),   // C2
            (36, 5.0, 0.3),   // C2
            (38, 5.5, 0.2),   // D2
            (40, 6.0, 0.3),   // E2
            (43, 6.5, 0.2),   // G2
            (48, 7.0, 0.8),   // C3
            (36, 8.0, 1.5),   // C2 (long)
        ],
    },
    Melody {
        name: "sustained",
        notes: &[
            // Just one long sustained chord
            (48, 0.0, 10.0), // C3
            (52, 0.0, 10.0), // E3
            (55, 0.0, 10.0), // G3
            (59, 0.0, 10.0), // B3
        ],
    },
];

/// Factory presets base path
const FACTORY_PRESETS_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../odin2/assets/Soundbanks/Factory Presets");

/// Interpolated preset parameters for real-time morphing
#[derive(Clone)]
struct MorphedParams {
    // Oscillator
    osc_detune: f32,
    osc_spread: f32,

    // Filter
    filter_freq: f32,
    filter_res: f32,
    filter_env_amount: f32,

    // Amp envelope
    amp_attack: f32,
    amp_decay: f32,
    amp_sustain: f32,
    amp_release: f32,

    // Filter envelope
    fil_attack: f32,
    fil_decay: f32,
    fil_sustain: f32,
    fil_release: f32,

    // Effects
    chorus_amount: f32,
    chorus_rate: f32,
    chorus_mix: f32,

    delay_time: f32,
    delay_feedback: f32,
    delay_mix: f32,

    reverb_mix: f32,

    // Master
    master: f32,
}

impl MorphedParams {
    fn from_preset(preset: &OdinPreset) -> Self {
        Self {
            osc_detune: preset.osc1.detune,
            osc_spread: preset.osc1.spread,

            filter_freq: preset.filter1.frequency,
            filter_res: preset.filter1.resonance,
            filter_env_amount: preset.filter1.env_amount,

            amp_attack: preset.env1.attack.max(0.001),
            amp_decay: preset.env1.decay.max(0.001),
            amp_sustain: preset.env1.sustain,
            amp_release: preset.env1.release.max(0.01),

            fil_attack: preset.env2.attack.max(0.001),
            fil_decay: preset.env2.decay.max(0.001),
            fil_sustain: preset.env2.sustain,
            fil_release: preset.env2.release.max(0.01),

            chorus_amount: if preset.chorus.on { preset.chorus.amount } else { 0.0 },
            chorus_rate: preset.chorus.rate,
            chorus_mix: if preset.chorus.on { preset.chorus.dry_wet } else { 0.0 },

            delay_time: preset.delay.time,
            delay_feedback: preset.delay.feedback,
            delay_mix: if preset.delay.on { preset.delay.wet } else { 0.0 },

            reverb_mix: if preset.reverb.on { preset.reverb.dry_wet } else { 0.0 },

            master: if preset.master > 0.0 { preset.master } else { 0.7 },
        }
    }

    /// Linear interpolation between two parameter sets
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;

        Self {
            osc_detune: a.osc_detune * inv_t + b.osc_detune * t,
            osc_spread: a.osc_spread * inv_t + b.osc_spread * t,

            filter_freq: a.filter_freq * inv_t + b.filter_freq * t,
            filter_res: a.filter_res * inv_t + b.filter_res * t,
            filter_env_amount: a.filter_env_amount * inv_t + b.filter_env_amount * t,

            amp_attack: a.amp_attack * inv_t + b.amp_attack * t,
            amp_decay: a.amp_decay * inv_t + b.amp_decay * t,
            amp_sustain: a.amp_sustain * inv_t + b.amp_sustain * t,
            amp_release: a.amp_release * inv_t + b.amp_release * t,

            fil_attack: a.fil_attack * inv_t + b.fil_attack * t,
            fil_decay: a.fil_decay * inv_t + b.fil_decay * t,
            fil_sustain: a.fil_sustain * inv_t + b.fil_sustain * t,
            fil_release: a.fil_release * inv_t + b.fil_release * t,

            chorus_amount: a.chorus_amount * inv_t + b.chorus_amount * t,
            chorus_rate: a.chorus_rate * inv_t + b.chorus_rate * t,
            chorus_mix: a.chorus_mix * inv_t + b.chorus_mix * t,

            delay_time: a.delay_time * inv_t + b.delay_time * t,
            delay_feedback: a.delay_feedback * inv_t + b.delay_feedback * t,
            delay_mix: a.delay_mix * inv_t + b.delay_mix * t,

            reverb_mix: a.reverb_mix * inv_t + b.reverb_mix * t,

            master: a.master * inv_t + b.master * t,
        }
    }

    /// Smooth interpolation (ease in/out) between two parameter sets
    fn smooth_lerp(a: &Self, b: &Self, t: f32) -> Self {
        // Smoothstep function for smoother transitions
        let t = t.clamp(0.0, 1.0);
        let smooth_t = t * t * (3.0 - 2.0 * t);
        Self::lerp(a, b, smooth_t)
    }
}

/// Morph sequence definition
struct MorphSequence {
    presets: Vec<OdinPreset>,
    params: Vec<MorphedParams>,
    durations: Vec<f32>, // Duration for each morph transition in seconds
}

impl MorphSequence {
    fn new() -> Self {
        Self {
            presets: Vec::new(),
            params: Vec::new(),
            durations: Vec::new(),
        }
    }

    fn add_preset(&mut self, preset: OdinPreset, transition_duration: f32) {
        self.params.push(MorphedParams::from_preset(&preset));
        self.presets.push(preset);
        if self.durations.len() < self.params.len() - 1 {
            // Duration for morphing FROM previous TO this preset
        }
        if !self.durations.is_empty() || self.params.len() > 1 {
            self.durations.push(transition_duration);
        }
    }

    fn total_duration(&self) -> f32 {
        self.durations.iter().sum()
    }

    /// Get interpolated parameters at a given time
    fn get_params_at(&self, time: f32) -> MorphedParams {
        if self.params.is_empty() {
            return MorphedParams::from_preset(&OdinPreset::default());
        }
        if self.params.len() == 1 {
            return self.params[0].clone();
        }

        let mut accumulated_time = 0.0;
        for (i, &duration) in self.durations.iter().enumerate() {
            if time < accumulated_time + duration {
                let local_t = (time - accumulated_time) / duration;
                return MorphedParams::smooth_lerp(&self.params[i], &self.params[i + 1], local_t);
            }
            accumulated_time += duration;
        }

        // Return last preset params if past all transitions
        self.params.last().unwrap().clone()
    }
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * libm::powf(2.0, (note as f32 - 69.0) / 12.0)
}

fn find_factory_preset(name: &str) -> Option<std::path::PathBuf> {
    let base_path = Path::new(FACTORY_PRESETS_PATH);
    if !base_path.exists() {
        return None;
    }

    let search_name = name.to_lowercase();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(files) = fs::read_dir(entry.path()) {
                    for file in files.flatten() {
                        if file.path().extension().map_or(false, |ext| ext == "odin") {
                            let preset_name = file
                                .path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string().to_lowercase())
                                .unwrap_or_default();

                            if preset_name.contains(&search_name) {
                                return Some(file.path());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn load_preset(name: &str) -> Option<OdinPreset> {
    find_factory_preset(name).and_then(|path| {
        println!("  Loading: {} -> {}", name, path.display());
        OdinPreset::load(&path).ok()
    })
}

/// Voice state for polyphonic rendering
struct Voice {
    osc: MultiOscillator,
    filter: LadderFilter,
    amp_env: Adsr,
    filter_env: Adsr,
    note: u8,
    active: bool,
    start_sample: usize,
    release_sample: usize,
}

impl Voice {
    fn new(sr: f32) -> Self {
        Self {
            osc: MultiOscillator::new(sr),
            filter: LadderFilter::new(sr),
            amp_env: Adsr::new(sr),
            filter_env: Adsr::new(sr),
            note: 0,
            active: false,
            start_sample: 0,
            release_sample: 0,
        }
    }

    fn note_on(&mut self, note: u8, start_sample: usize, duration_samples: usize, params: &MorphedParams) {
        self.note = note;
        self.osc.set_frequency(midi_to_freq(note));
        self.osc.set_wavetable(1);
        self.osc.randomize_phase();

        // Set envelope from morphed params
        self.amp_env.set_attack(params.amp_attack);
        self.amp_env.set_decay(params.amp_decay);
        self.amp_env.set_sustain(params.amp_sustain);
        self.amp_env.set_release(params.amp_release);
        self.amp_env.trigger();

        self.filter_env.set_attack(params.fil_attack);
        self.filter_env.set_decay(params.fil_decay);
        self.filter_env.set_sustain(params.fil_sustain);
        self.filter_env.set_release(params.fil_release);
        self.filter_env.trigger();

        self.active = true;
        self.start_sample = start_sample;
        self.release_sample = start_sample + duration_samples;
    }

    fn process(&mut self, params: &MorphedParams) -> (f32, f32) {
        if !self.active {
            return (0.0, 0.0);
        }

        // Update oscillator with morphed params
        self.osc.set_detune(params.osc_detune);
        self.osc.set_stereo_width(params.osc_spread);

        // Update filter with morphed params + envelope
        let filter_env_val = self.filter_env.process();
        let mod_cutoff = params.filter_freq + filter_env_val * params.filter_env_amount * 5000.0;
        self.filter.set_cutoff(mod_cutoff.clamp(20.0, 20000.0));
        self.filter.set_resonance(params.filter_res);
        self.filter.set_filter_type(LadderFilterType::LP4);

        let amp_env_val = self.amp_env.process();

        // Check if voice finished
        if amp_env_val < 0.0001 && !self.amp_env.is_active() {
            self.active = false;
            return (0.0, 0.0);
        }

        let (l, r) = self.osc.process_stereo();
        let filtered_l = self.filter.process(l);
        let filtered_r = self.filter.process(r);

        (filtered_l * amp_env_val, filtered_r * amp_env_val)
    }

    fn check_release(&mut self, current_sample: usize) {
        if self.active && current_sample >= self.release_sample {
            self.amp_env.release();
            self.filter_env.release();
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

/// Generate morphing audio with a melody
fn generate_morph_audio(sequence: &MorphSequence, melody: &Melody) -> Vec<f32> {
    let sr = SAMPLE_RATE as f32;
    let total_duration = sequence.total_duration();
    let num_samples = (sr * total_duration) as usize;
    let mut samples = vec![0.0f32; num_samples * 2];

    println!("  Generating {:.1}s of morphing audio with melody '{}'...", total_duration, melody.name);

    // Voice pool for polyphony
    const MAX_VOICES: usize = 16;
    let mut voices: Vec<Voice> = (0..MAX_VOICES).map(|_| Voice::new(sr)).collect();

    // Schedule notes - scale timing to fit morph duration
    let melody_duration = melody.notes.iter()
        .map(|&(_, start, dur)| start + dur)
        .fold(0.0f32, |a, b| a.max(b));

    let time_scale = if melody_duration > 0.0 { total_duration / melody_duration } else { 1.0 };

    // Convert melody to sample-based events
    let note_events: Vec<(usize, usize, u8)> = melody.notes.iter()
        .map(|&(note, start, dur)| {
            let start_sample = (start * time_scale * sr) as usize;
            let duration_samples = (dur * time_scale * sr) as usize;
            (start_sample, duration_samples, note)
        })
        .collect();

    // Effects
    let mut chorus = Chorus::new(sr);
    let mut delay = Delay::new(sr);
    let mut reverb = ZitaReverb::new(sr);

    // Process each sample
    for i in 0..num_samples {
        let time = i as f32 / sr;
        let params = sequence.get_params_at(time);

        // Trigger new notes
        for &(start_sample, duration_samples, note) in &note_events {
            if i == start_sample {
                // Find free voice
                if let Some(voice) = voices.iter_mut().find(|v| !v.is_active()) {
                    voice.note_on(note, start_sample, duration_samples, &params);
                }
            }
        }

        // Check for note releases
        for voice in &mut voices {
            voice.check_release(i);
        }

        // Update effects with morphed parameters
        chorus.set_amount(params.chorus_amount);
        chorus.set_lfo_freq(params.chorus_rate.max(0.1));
        chorus.set_dry_wet(params.chorus_mix);

        delay.set_delay_time(params.delay_time.max(0.01));
        delay.set_feedback(params.delay_feedback);
        delay.set_wet(params.delay_mix);
        delay.set_dry(1.0 - params.delay_mix * 0.5);

        reverb.set_mix(params.reverb_mix);

        // Generate and mix all active voices
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        let mut active_count = 0;

        for voice in &mut voices {
            if voice.is_active() {
                let (l, r) = voice.process(&params);
                left += l;
                right += r;
                active_count += 1;
            }
        }

        // Normalize by active voice count
        if active_count > 0 {
            let voice_scale = 1.0 / (active_count as f32).sqrt();
            left *= voice_scale;
            right *= voice_scale;
        }

        // Apply chorus
        if params.chorus_mix > 0.01 {
            let mono = (left + right) * 0.5;
            let (cl, cr) = chorus.process(mono);
            left = left * (1.0 - params.chorus_mix) + cl * params.chorus_mix;
            right = right * (1.0 - params.chorus_mix) + cr * params.chorus_mix;
        }

        // Apply delay
        if params.delay_mix > 0.01 {
            let (dl, dr) = delay.process(left, right);
            left = dl;
            right = dr;
        }

        // Apply reverb
        if params.reverb_mix > 0.01 {
            let (rl, rr) = reverb.process(left, right);
            left = rl;
            right = rr;
        }

        // Master volume
        left *= params.master * 0.4;
        right *= params.master * 0.4;

        samples[i * 2] = left;
        samples[i * 2 + 1] = right;
    }

    samples
}

fn write_stereo_wav(path: &str, samples: &[f32]) {
    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let spec = WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).expect("Failed to create WAV file");

    for &sample in samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_sample(sample_i16).unwrap();
    }

    writer.finalize().unwrap();
}

fn print_usage() {
    println!("Odin 2 Preset Morphing Demo");
    println!();
    println!("Usage: morph-demo [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --demo N            Run built-in demo N (1-3)");
    println!("  --presets A,B,C     Morph between named presets (comma-separated)");
    println!("  --melody NAME       Melody to play (default: slow_arpeggio)");
    println!("  --duration SECS     Duration per morph transition (default: 4.0)");
    println!("  --output FILE       Output WAV file (default: ../../samples/demos/morph_output.wav)");
    println!("  --play              Play audio after generation");
    println!("  --help              Show this help");
    println!();
    println!("Built-in Demos:");
    println!("  1: Pad Evolution    - Warm Pad → Synthwave Pad → Eerie Pad");
    println!("  2: Soft to Harsh    - Synth Strings → Lead Sine → Lead Square");
    println!("  3: Atmospheric      - Frosty Atmo → Drifter → Moonshade");
    println!();
    println!("Available Melodies:");
    println!("  slow_arpeggio  - Ascending/descending arpeggio (12 notes)");
    println!("  chord_stabs    - Chord progression stabs (C-F-Am-G-C)");
    println!("  lead_line      - Melodic lead line (15 notes)");
    println!("  bass_groove    - Bass line groove (16 notes)");
    println!("  sustained      - One long sustained chord");
    println!();
    println!("Examples:");
    println!("  morph-demo --demo 1 --play");
    println!("  morph-demo --demo 1 --melody lead_line --play");
    println!("  morph-demo --presets \"Warm-Pad,Lead Sine\" --melody chord_stabs --play");
    println!("  morph-demo --presets \"Analog Bass,Lead Rusty\" --melody bass_groove --duration 5 --play");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut demo_num: Option<u32> = None;
    let mut preset_names: Option<String> = None;
    let mut melody_name = "slow_arpeggio".to_string();
    let mut duration = 4.0f32;
    let mut output_file = "../../samples/demos/morph_output.wav".to_string();
    let mut play_audio = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--demo" | "-d" => {
                i += 1;
                if i < args.len() {
                    demo_num = args[i].parse().ok();
                }
            }
            "--presets" | "-p" => {
                i += 1;
                if i < args.len() {
                    preset_names = Some(args[i].clone());
                }
            }
            "--melody" | "-m" => {
                i += 1;
                if i < args.len() {
                    melody_name = args[i].clone();
                }
            }
            "--duration" | "-t" => {
                i += 1;
                if i < args.len() {
                    duration = args[i].parse().unwrap_or(4.0);
                }
            }
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output_file = args[i].clone();
                }
            }
            "--play" => {
                play_audio = true;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                print_usage();
                return;
            }
        }
        i += 1;
    }

    // Determine which presets to load
    let preset_list: Vec<&str> = if let Some(ref names) = preset_names {
        names.split(',').map(|s| s.trim()).collect()
    } else if let Some(num) = demo_num {
        match num {
            1 => vec!["Warm-Pad", "Synthwave Pad", "Eerie Pad"],
            2 => vec!["Synth Strings", "Lead Sine", "Lead Square"],
            3 => vec!["Frosty Atmo", "Drifter", "Moonshade"],
            _ => {
                eprintln!("Unknown demo number: {}. Use 1, 2, or 3.", num);
                return;
            }
        }
    } else {
        // Default demo
        println!("No demo or presets specified. Running Demo 1 (Pad Evolution).");
        println!();
        vec!["Warm-Pad", "Synthwave Pad", "Eerie Pad"]
    };

    if preset_list.len() < 2 {
        eprintln!("Need at least 2 presets for morphing.");
        return;
    }

    println!("=== Odin 2 Preset Morphing Demo ===");
    println!();
    println!("Loading presets:");

    let mut sequence = MorphSequence::new();
    for (i, name) in preset_list.iter().enumerate() {
        match load_preset(name) {
            Some(preset) => {
                println!("    ✓ '{}' ({:?}, filter @ {:.0} Hz)",
                         preset.name, preset.osc1.osc_type, preset.filter1.frequency);
                let trans_duration = if i == 0 { 0.0 } else { duration };
                sequence.add_preset(preset, trans_duration);
            }
            None => {
                eprintln!("    ✗ Preset '{}' not found!", name);
                return;
            }
        }
    }

    // Find melody
    let melody = MELODIES
        .iter()
        .find(|m| m.name == melody_name)
        .unwrap_or_else(|| {
            eprintln!("Unknown melody: '{}'. Using 'slow_arpeggio'.", melody_name);
            &MELODIES[0]
        });

    println!();
    println!("Morph sequence:");
    for (i, preset) in sequence.presets.iter().enumerate() {
        if i > 0 {
            println!("    ↓ ({:.1}s morph)", sequence.durations[i - 1]);
        }
        println!("  [{}] {}", i + 1, preset.name);
    }
    println!();
    println!("Total duration: {:.1}s", sequence.total_duration());
    println!("Melody: {} ({} notes)", melody.name, melody.notes.len());
    println!();

    let samples = generate_morph_audio(&sequence, melody);

    println!("  Writing to: {}", output_file);
    write_stereo_wav(&output_file, &samples);

    println!();
    println!("Done!");

    if play_audio {
        println!();
        println!("Playing audio...");
        let status = Command::new("afplay").arg(&output_file).status();

        match status {
            Ok(s) if s.success() => {}
            Ok(_) => eprintln!("Warning: afplay exited with non-zero status"),
            Err(e) => {
                eprintln!("Error: Could not play audio: {}", e);
                eprintln!("Note: --play requires macOS with afplay command");
            }
        }
    }
}
