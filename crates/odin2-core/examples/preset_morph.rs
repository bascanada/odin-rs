//! Preset Morphing Example
//!
//! Demonstrates smooth morphing between two Odin 2 presets.
//!
//! Usage:
//!   cargo run --example preset_morph --features std
//!   cargo run --example preset_morph --features std -- --generate-audio

use odin2_core::preset::OdinPreset;
use odin2_core::engine::{OdinEngine, SynthEngine};
use hound::{WavSpec, WavWriter};
use std::env;
use std::process::Command;

fn main() {
    println!("=== Odin 2 Preset Morphing Example ===\n");

    // Check if we should generate audio files
    let args: Vec<String> = env::args().collect();
    let generate_audio = args.iter().any(|arg| arg == "--generate-audio");

    // Create engine
    let sample_rate = 44100.0;
    let mut engine = OdinEngine::new(sample_rate);

    // Create procedural emotional presets (no files needed!)
    println!("Creating procedural emotional presets...\n");

    let preset_happy = OdinPreset::create_happy();
    println!("✓ Created: {} (osc1 vol: {:.2}, filter: {:.0} Hz)",
             preset_happy.name, preset_happy.osc1.volume, preset_happy.filter1.frequency);

    let preset_sad = OdinPreset::create_sad();
    println!("✓ Created: {} (osc1 vol: {:.2}, filter: {:.0} Hz)",
             preset_sad.name, preset_sad.osc1.volume, preset_sad.filter1.frequency);

    let preset_angry = OdinPreset::create_angry();
    println!("✓ Created: {} (osc1 vol: {:.2}, filter: {:.0} Hz)",
             preset_angry.name, preset_angry.osc1.volume, preset_angry.filter1.frequency);

    let preset_calm = OdinPreset::create_calm();
    println!("✓ Created: {} (osc1 vol: {:.2}, filter: {:.0} Hz)",
             preset_calm.name, preset_calm.osc1.volume, preset_calm.filter1.frequency);

    let preset_a = preset_happy;
    let preset_b = preset_sad;

    println!("\n--- Morphing Examples ---\n");

    // Example 1: Pure preset A (t=0.0)
    println!("1. Pure Preset A (t=0.0):");
    let morphed_0 = preset_a.interpolate(&preset_b, 0.0);
    println!("   Name: {}", morphed_0.name);
    println!("   Osc1 volume: {:.3}", morphed_0.osc1.volume);
    println!("   Filter freq: {:.1} Hz", morphed_0.filter1.frequency);
    println!("   Env1 attack: {:.3}s", morphed_0.env1.attack);

    // Example 2: 30% toward preset B (t=0.3)
    println!("\n2. 30% Toward Preset B (t=0.3):");
    let morphed_30 = preset_a.interpolate(&preset_b, 0.3);
    println!("   Name: {}", morphed_30.name);
    println!("   Osc1 volume: {:.3}", morphed_30.osc1.volume);
    println!("   Filter freq: {:.1} Hz", morphed_30.filter1.frequency);
    println!("   Env1 attack: {:.3}s", morphed_30.env1.attack);

    // Example 3: Halfway blend (t=0.5)
    println!("\n3. Halfway Blend (t=0.5):");
    let morphed_50 = preset_a.interpolate(&preset_b, 0.5);
    println!("   Name: {}", morphed_50.name);
    println!("   Osc1 volume: {:.3}", morphed_50.osc1.volume);
    println!("   Filter freq: {:.1} Hz", morphed_50.filter1.frequency);
    println!("   Env1 attack: {:.3}s", morphed_50.env1.attack);

    // Example 4: Smooth interpolation
    println!("\n4. Smooth Interpolation (t=0.5 with easing):");
    let morphed_smooth = preset_a.interpolate_smooth(&preset_b, 0.5);
    println!("   Name: {}", morphed_smooth.name);
    println!("   Osc1 volume: {:.3}", morphed_smooth.osc1.volume);

    // Example 5: Pure preset B (t=1.0)
    println!("\n5. Pure Preset B (t=1.0):");
    let morphed_100 = preset_a.interpolate(&preset_b, 1.0);
    println!("   Name: {}", morphed_100.name);
    println!("   Osc1 volume: {:.3}", morphed_100.osc1.volume);

    // Load morphed preset into engine
    println!("\n--- Loading Morphed Preset into Engine ---");
    engine.load_preset(&morphed_50);
    println!("✓ Loaded 50% blend into engine");
    println!("  Master volume: {:.2}", engine.master_volume);

    // Generate audio if requested
    if generate_audio {
        println!("\n--- Generating Audio Files ---");
        generate_audio_files(sample_rate);
    } else {
        // Quick test
        println!("\n--- Quick Audio Test ---");
        println!("Playing note C4 (MIDI 60) for 1000 samples...");

        engine.note_on(60, 100);
        let mut buffer = vec![0.0f32; 2000]; // 1000 stereo samples
        engine.process(&mut buffer, 2);

        // Check if we got audio output
        let max_amplitude = buffer.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        println!("Max amplitude: {:.4}", max_amplitude);

        if max_amplitude > 0.0001 {
            println!("✓ Audio generated successfully!");
        } else {
            println!("⚠ Warning: Audio output is very quiet or silent");
        }

        println!("\n💡 Tip: Run with --generate-audio to create WAV files you can listen to:");
        println!("   cargo run --example preset_morph --features std -- --generate-audio");
    }

    println!("\n--- 2D Emotional Space Example ---");
    println!("\nUsing bilinear interpolation in 2D emotion space:");
    println!("  Valence (x-axis): 0.0 (negative) → 1.0 (positive)");
    println!("  Arousal (y-axis): 0.0 (low energy) → 1.0 (high energy)\n");

    // Corner presets
    println!("Corner positions:");
    println!("  (0.0, 0.0) = Sad     (low valence, low arousal)");
    println!("  (1.0, 0.0) = Calm    (high valence, low arousal)");
    println!("  (0.0, 1.0) = Angry   (low valence, high arousal)");
    println!("  (1.0, 1.0) = Happy   (high valence, high arousal)");

    // Test some 2D positions
    println!("\nTest positions:");

    let pos1 = OdinPreset::create_emotional_2d(0.5, 0.5);
    println!("  (0.5, 0.5) - Neutral: {}", pos1.name);

    let pos2 = OdinPreset::create_emotional_2d(0.8, 0.3);
    println!("  (0.8, 0.3) - Peaceful: {}", pos2.name);

    let pos3 = OdinPreset::create_emotional_2d(0.2, 0.7);
    println!("  (0.2, 0.7) - Anxious: {}", pos3.name);

    println!("\n--- Usage in Your Game ---");
    println!("\nFor procedural music generation:");
    println!("```rust");
    println!("// Create presets (once at startup)");
    println!("let happy = OdinPreset::create_happy();");
    println!("let sad = OdinPreset::create_sad();");
    println!();
    println!("// In your game loop:");
    println!("let emotion = player_emotion_score(); // 0.0 to 1.0");
    println!("let sound = happy.interpolate(&sad, emotion);");
    println!("engine.load_preset(&sound);");
    println!();
    println!("// Or use 2D emotional space:");
    println!("let valence = player_happiness();  // 0.0 to 1.0");
    println!("let arousal = player_energy();     // 0.0 to 1.0");
    println!("let sound = OdinPreset::create_emotional_2d(valence, arousal);");
    println!("engine.load_preset(&sound);");
    println!("```");

    println!("\n✓ Done!");
}

/// Generate audio files demonstrating preset morphing
fn generate_audio_files(sample_rate: f32) {
    // Create output directory
    let output_dir = "../../samples/morphing";
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let happy = OdinPreset::create_happy();
    let sad = OdinPreset::create_sad();

    // Melody to play
    let melody = [
        (60, 0.0, 0.5),   // C4
        (64, 0.5, 0.5),   // E4
        (67, 1.0, 0.5),   // G4
        (72, 1.5, 0.5),   // C5
        (67, 2.0, 0.5),   // G4
        (64, 2.5, 0.5),   // E4
        (60, 3.0, 1.0),   // C4 (long)
    ];

    let morph_levels = [
        (0.0, "00_pure_happy"),
        (0.25, "25_mostly_happy"),
        (0.5, "50_halfway"),
        (0.75, "75_mostly_sad"),
        (1.0, "100_pure_sad"),
    ];

    for (t, filename) in &morph_levels {
        println!("Generating: morph_{}.wav (t={:.2})", filename, t);

        let morphed = happy.interpolate(&sad, *t);
        let audio = render_melody(&morphed, &melody, sample_rate);

        let wav_path = format!("{}/morph_{}.wav", output_dir, filename);
        write_wav(&wav_path, &audio, sample_rate as u32);
        println!("  ✓ Saved to {}", wav_path);
    }

    println!("\n--- 2D Emotional Space Demos ---");

    let positions_2d = [
        ((1.0, 1.0), "2d_happy"),
        ((0.0, 0.0), "2d_sad"),
        ((0.0, 1.0), "2d_angry"),
        ((1.0, 0.0), "2d_calm"),
        ((0.5, 0.5), "2d_neutral"),
    ];

    for ((valence, arousal), filename) in &positions_2d {
        println!("Generating: {}.wav (v={:.1}, a={:.1})", filename, valence, arousal);

        let preset = OdinPreset::create_emotional_2d(*valence, *arousal);
        let audio = render_melody(&preset, &melody, sample_rate);

        let wav_path = format!("{}/{}.wav", output_dir, filename);
        write_wav(&wav_path, &audio, sample_rate as u32);
        println!("  ✓ Saved to {}", wav_path);
    }

    println!("\n✅ All audio files generated in: {}/", output_dir);
    println!("\nYou can now listen to:");
    println!("  - {}/morph_00_pure_happy.wav → morph_100_pure_sad.wav", output_dir);
    println!("  - {}/2d_happy.wav, 2d_sad.wav, 2d_angry.wav, 2d_calm.wav", output_dir);

    // Try to play the first file on macOS
    #[cfg(target_os = "macos")]
    {
        let play_file = format!("{}/morph_50_halfway.wav", output_dir);
        println!("\n🔊 Playing {}...", play_file);
        let _ = Command::new("afplay").arg(&play_file).status();
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("\n💡 Use your audio player to listen to the generated WAV files.");
    }
}

/// Render a melody with a given preset
fn render_melody(
    preset: &OdinPreset,
    melody: &[(u8, f32, f32)], // (note, start_time, duration)
    sample_rate: f32,
) -> Vec<f32> {
    let mut engine = OdinEngine::new(sample_rate);
    engine.load_preset(preset);

    // Calculate total duration
    let total_duration = melody.iter()
        .map(|(_, start, dur)| start + dur)
        .fold(0.0f32, |a, b| a.max(b));

    let num_samples = (sample_rate * total_duration) as usize;
    let mut buffer = vec![0.0f32; num_samples * 2]; // Stereo

    // Convert melody to sample-based events
    let note_events: Vec<(u8, usize, usize)> = melody.iter()
        .map(|(note, start, dur)| {
            let start_sample = (start * sample_rate) as usize;
            let duration_samples = (dur * sample_rate) as usize;
            (*note, start_sample, duration_samples)
        })
        .collect();

    // Track active notes for note-off
    let mut active_notes: Vec<(u8, usize)> = Vec::new(); // (note, release_sample)

    for i in 0..num_samples {
        // Trigger new notes
        for &(note, start_sample, duration_samples) in &note_events {
            if i == start_sample {
                engine.note_on(note, 100);
                active_notes.push((note, start_sample + duration_samples));
            }
        }

        // Release notes
        active_notes.retain(|(note, release_sample)| {
            if i >= *release_sample {
                engine.note_off(*note);
                false
            } else {
                true
            }
        });

        // Process one sample
        let mut sample_buf = [0.0f32, 0.0f32];
        engine.process(&mut sample_buf, 2);

        buffer[i * 2] = sample_buf[0];
        buffer[i * 2 + 1] = sample_buf[1];
    }

    buffer
}

/// Write stereo audio to WAV file
fn write_wav(path: &str, samples: &[f32], sample_rate: u32) {
    let spec = WavSpec {
        channels: 2,
        sample_rate,
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
