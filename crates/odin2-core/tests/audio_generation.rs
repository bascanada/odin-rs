//! Audio generation tests
//!
//! These tests generate WAV files for auditory validation of the DSP modules.

use odin2_core::dsp::oscillators::{AnalogOscillator, Oscillator, Waveform};
use odin2_core::dsp::filters::{LadderFilter, Filter, LadderFilterType};
use odin2_core::dsp::envelopes::{Adsr, Envelope};
use odin2_core::engine::{OdinEngine, SynthEngine};

use hound::{WavSpec, WavWriter};

const SAMPLE_RATE: u32 = 44100;

fn write_wav(path: &str, samples: &[f32]) {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();

    for &sample in samples {
        // Clamp and convert to i16
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_sample(sample_i16).unwrap();
    }

    writer.finalize().unwrap();
}

fn write_stereo_wav(path: &str, samples: &[f32]) {
    let spec = WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();

    for &sample in samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_sample(sample_i16).unwrap();
    }

    writer.finalize().unwrap();
}

#[test]
fn test_generate_oscillator_waveforms() {
    let duration_secs = 2.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Generate each waveform
    for waveform in [Waveform::Saw, Waveform::Square, Waveform::Triangle, Waveform::Sine] {
        let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
        osc.set_waveform(waveform);
        osc.set_frequency(440.0); // A4

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(osc.process() * 0.5); // -6dB to avoid clipping
        }

        let filename = format!("target/test_osc_{:?}.wav", waveform).to_lowercase();
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_filter_sweep() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(110.0); // A2

    let mut filter = LadderFilter::new(SAMPLE_RATE as f32);
    filter.set_filter_type(LadderFilterType::LP4);
    filter.set_resonance(0.7);

    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        // Sweep cutoff from 100 Hz to 10000 Hz
        let t = i as f32 / num_samples as f32;
        let cutoff = 100.0 * (100.0_f32).powf(t); // Exponential sweep

        filter.set_cutoff(cutoff);

        let osc_out = osc.process();
        let filtered = filter.process(osc_out);
        samples.push(filtered * 0.5);
    }

    write_wav("target/test_filter_sweep.wav", &samples);
    println!("Generated: target/test_filter_sweep.wav");
}

#[test]
fn test_generate_adsr_envelope() {
    let duration_secs = 3.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(440.0);

    let mut env = Adsr::new(SAMPLE_RATE as f32);
    env.set_attack(0.1);   // 100ms attack
    env.set_decay(0.2);    // 200ms decay
    env.set_sustain(0.6);  // 60% sustain
    env.set_release(0.5);  // 500ms release

    let mut samples = Vec::with_capacity(num_samples);

    // Trigger at start
    env.trigger();

    // Release after 1.5 seconds
    let release_sample = (SAMPLE_RATE as f32 * 1.5) as usize;

    for i in 0..num_samples {
        if i == release_sample {
            env.release();
        }

        let env_val = env.process();
        let osc_out = osc.process();
        samples.push(osc_out * env_val * 0.5);
    }

    write_wav("target/test_adsr.wav", &samples);
    println!("Generated: target/test_adsr.wav");
}

#[test]
fn test_generate_synth_melody() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut engine = OdinEngine::new(SAMPLE_RATE as f32);

    // Simple melody: C4, E4, G4, C5
    let notes = [
        (60, 0.0, 0.4),   // C4 at 0s, duration 0.4s
        (64, 0.5, 0.4),   // E4 at 0.5s
        (67, 1.0, 0.4),   // G4 at 1.0s
        (72, 1.5, 0.8),   // C5 at 1.5s, longer
        (67, 2.5, 0.4),   // G4 at 2.5s
        (64, 3.0, 0.4),   // E4 at 3.0s
        (60, 3.5, 0.4),   // C4 at 3.5s
    ];

    let mut samples = vec![0.0f32; num_samples * 2]; // Stereo

    // Process in blocks
    let block_size = 64;
    let mut block = vec![0.0f32; block_size * 2];

    for block_start in (0..num_samples).step_by(block_size) {
        let block_end = (block_start + block_size).min(num_samples);
        let actual_block_size = block_end - block_start;

        // Check for note events in this block
        let block_start_time = block_start as f32 / SAMPLE_RATE as f32;
        let block_end_time = block_end as f32 / SAMPLE_RATE as f32;

        for &(note, start_time, duration) in &notes {
            // Note on
            if start_time >= block_start_time && start_time < block_end_time {
                engine.note_on(note, 100);
            }
            // Note off
            let end_time = start_time + duration;
            if end_time >= block_start_time && end_time < block_end_time {
                engine.note_off(note);
            }
        }

        // Process audio
        block.fill(0.0);
        engine.process(&mut block[..actual_block_size * 2], 2);

        // Copy to output
        for i in 0..actual_block_size * 2 {
            samples[block_start * 2 + i] = block[i];
        }
    }

    write_stereo_wav("target/test_synth_melody.wav", &samples);
    println!("Generated: target/test_synth_melody.wav");
}

#[test]
fn test_generate_chord() {
    let duration_secs = 2.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut engine = OdinEngine::new(SAMPLE_RATE as f32);

    // C major chord
    engine.note_on(60, 80);  // C4
    engine.note_on(64, 80);  // E4
    engine.note_on(67, 80);  // G4

    let mut samples = vec![0.0f32; num_samples * 2];

    // Process in blocks
    let block_size = 256;
    let mut block = vec![0.0f32; block_size * 2];

    // Release after 1.5 seconds
    let release_sample = (SAMPLE_RATE as f32 * 1.5) as usize;
    let mut released = false;

    for block_start in (0..num_samples).step_by(block_size) {
        let block_end = (block_start + block_size).min(num_samples);
        let actual_block_size = block_end - block_start;

        if !released && block_start >= release_sample {
            engine.note_off(60);
            engine.note_off(64);
            engine.note_off(67);
            released = true;
        }

        block.fill(0.0);
        engine.process(&mut block[..actual_block_size * 2], 2);

        for i in 0..actual_block_size * 2 {
            samples[block_start * 2 + i] = block[i];
        }
    }

    write_stereo_wav("target/test_chord.wav", &samples);
    println!("Generated: target/test_chord.wav");
}
