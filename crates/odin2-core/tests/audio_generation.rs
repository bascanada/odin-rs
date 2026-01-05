//! Audio generation tests
//!
//! These tests generate WAV files for auditory validation of the DSP modules.

use odin2_core::dsp::oscillators::{
    AnalogOscillator, WavetableOscillator, Oscillator, Waveform,
    FmOscillator, FmMode, PmOscillator, Lfo, LfoWaveform, NoiseOscillator,
    MultiOscillator, DriftGenerator,
};
use odin2_core::dsp::filters::{LadderFilter, Filter, LadderFilterType, DiodeFilter, SemFilter, BiquadEQ, EQBand, EQBandType};
use odin2_core::dsp::envelopes::{Adsr, Envelope};
use odin2_core::dsp::effects::{Delay, Chorus, OversamplingDistortion, DistortionAlgorithm, Bitcrusher, ParametricEQ};
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

#[test]
fn test_generate_wavetable_demo() {
    let duration_secs = 2.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Generate demo of first 8 wavetables
    let wavetables_to_test = [0, 1, 2, 3, 4, 5, 6, 7];

    for &wt_index in &wavetables_to_test {
        let mut osc = WavetableOscillator::new(SAMPLE_RATE as f32);
        osc.set_wavetable(wt_index);
        osc.set_frequency(220.0); // A3

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(osc.process() * 0.5);
        }

        let name = osc.wavetable_name();
        let filename = format!("target/test_wavetable_{:03}_{}.wav", wt_index, name);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_wavetable_sweep() {
    let duration_secs = 8.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = WavetableOscillator::new(SAMPLE_RATE as f32);
    osc.set_wavetable(1); // FatSaw

    let mut filter = LadderFilter::new(SAMPLE_RATE as f32);
    filter.set_filter_type(LadderFilterType::LP4);
    filter.set_resonance(0.6);

    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / num_samples as f32;

        // Frequency sweep from C2 to C5
        let freq = 65.41 * (8.0_f32).powf(t);
        osc.set_frequency(freq);

        // Filter follows frequency
        filter.set_cutoff(freq * 4.0);

        let osc_out = osc.process();
        let filtered = filter.process(osc_out);
        samples.push(filtered * 0.5);
    }

    write_wav("target/test_wavetable_sweep.wav", &samples);
    println!("Generated: target/test_wavetable_sweep.wav");
}

#[test]
fn test_generate_fm_demo() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Test different FM ratios
    let ratios = [(1, 1), (1, 2), (2, 1), (1, 3), (3, 1), (2, 3)];

    for (carrier, modulator) in ratios {
        let mut osc = FmOscillator::new(SAMPLE_RATE as f32);
        osc.set_frequency(220.0); // A3
        osc.set_ratios(carrier, modulator);
        osc.set_fm_amount(0.5);
        osc.set_carrier_wavetable(0); // Sine carrier
        osc.set_modulator_wavetable(0); // Sine modulator
        osc.set_fm_mode(FmMode::Linear);

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(osc.process() * 0.5);
        }

        let filename = format!("target/test_fm_{}_{}.wav", carrier, modulator);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_fm_modulation_sweep() {
    let duration_secs = 6.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = FmOscillator::new(SAMPLE_RATE as f32);
    osc.set_frequency(220.0);
    osc.set_ratios(1, 2);
    osc.set_carrier_wavetable(0); // Sine
    osc.set_modulator_wavetable(0); // Sine
    osc.set_fm_mode(FmMode::Linear);

    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        // Sweep FM amount from 0 to 1
        let t = i as f32 / num_samples as f32;
        osc.set_fm_amount(t);

        samples.push(osc.process() * 0.5);
    }

    write_wav("target/test_fm_modulation_sweep.wav", &samples);
    println!("Generated: target/test_fm_modulation_sweep.wav");
}

#[test]
fn test_generate_fm_exponential() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = FmOscillator::new(SAMPLE_RATE as f32);
    osc.set_frequency(220.0);
    osc.set_ratios(1, 1);
    osc.set_fm_amount(0.4);
    osc.set_carrier_wavetable(0); // Sine
    osc.set_modulator_wavetable(0); // Sine
    osc.set_fm_mode(FmMode::Exponential);

    let mut samples = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        samples.push(osc.process() * 0.5);
    }

    write_wav("target/test_fm_exponential.wav", &samples);
    println!("Generated: target/test_fm_exponential.wav");
}

#[test]
fn test_generate_pm_demo() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Test different PM ratios
    let ratios = [(1, 1), (1, 2), (2, 1), (1, 3), (3, 1), (2, 3)];

    for (carrier, modulator) in ratios {
        let mut osc = PmOscillator::new(SAMPLE_RATE as f32);
        osc.set_frequency(220.0); // A3
        osc.set_ratios(carrier, modulator);
        osc.set_pm_amount(0.5);
        osc.set_carrier_wavetable(0); // Sine carrier
        osc.set_modulator_wavetable(0); // Sine modulator

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(osc.process() * 0.5);
        }

        let filename = format!("target/test_pm_{}_{}.wav", carrier, modulator);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_pm_modulation_sweep() {
    let duration_secs = 6.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = PmOscillator::new(SAMPLE_RATE as f32);
    osc.set_frequency(220.0);
    osc.set_ratios(1, 2);
    osc.set_carrier_wavetable(0); // Sine
    osc.set_modulator_wavetable(0); // Sine

    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        // Sweep PM amount from 0 to 1
        let t = i as f32 / num_samples as f32;
        osc.set_pm_amount(t);

        samples.push(osc.process() * 0.5);
    }

    write_wav("target/test_pm_modulation_sweep.wav", &samples);
    println!("Generated: target/test_pm_modulation_sweep.wav");
}

#[test]
fn test_generate_fm_wavetable_combo() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // FM with different wavetable combinations
    let combos = [
        (0, 0, "sine_sine"),
        (0, 1, "sine_fatsaw"),
        (1, 0, "fatsaw_sine"),
        (1, 1, "fatsaw_fatsaw"),
    ];

    for (carrier_wt, mod_wt, name) in combos {
        let mut osc = FmOscillator::new(SAMPLE_RATE as f32);
        osc.set_frequency(220.0);
        osc.set_ratios(1, 2);
        osc.set_fm_amount(0.4);
        osc.set_wavetables(carrier_wt, mod_wt);
        osc.set_fm_mode(FmMode::Linear);

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(osc.process() * 0.5);
        }

        let filename = format!("target/test_fm_wt_{}.wav", name);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_noise_filtered() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // White noise
    let mut noise = NoiseOscillator::new(SAMPLE_RATE as f32);
    noise.set_filter_freqs(20000.0, 20.0);

    let mut samples = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        samples.push(noise.process() * 0.3);
    }
    write_wav("target/test_noise_white.wav", &samples);
    println!("Generated: target/test_noise_white.wav");

    // Pink-ish noise (lowpass at 2kHz)
    noise.reset();
    noise.set_filter_freqs(2000.0, 20.0);
    samples.clear();
    for _ in 0..num_samples {
        samples.push(noise.process() * 0.3);
    }
    write_wav("target/test_noise_pink.wav", &samples);
    println!("Generated: target/test_noise_pink.wav");

    // Bandpass noise
    noise.reset();
    noise.set_filter_freqs(3000.0, 500.0);
    samples.clear();
    for _ in 0..num_samples {
        samples.push(noise.process() * 0.5);
    }
    write_wav("target/test_noise_bandpass.wav", &samples);
    println!("Generated: target/test_noise_bandpass.wav");
}

#[test]
fn test_generate_diode_filter() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(110.0);

    let mut filter = DiodeFilter::new(SAMPLE_RATE as f32);
    filter.set_resonance(0.7);

    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / num_samples as f32;
        let cutoff = 100.0 * (100.0_f32).powf(t);
        filter.set_cutoff(cutoff);

        let osc_out = osc.process();
        let filtered = filter.process(osc_out);
        samples.push(filtered * 0.5);
    }

    write_wav("target/test_diode_sweep.wav", &samples);
    println!("Generated: target/test_diode_sweep.wav");
}

#[test]
fn test_generate_sem_filter() {
    let duration_secs = 6.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(110.0);

    let mut filter = SemFilter::new(SAMPLE_RATE as f32);
    filter.set_cutoff(800.0);
    filter.set_resonance(0.7);

    let mut samples = Vec::with_capacity(num_samples);

    // Sweep transition from LP to HP
    for i in 0..num_samples {
        let t = i as f32 / num_samples as f32;
        // Transition from -1 (LP) to +1 (HP)
        let transition = t * 2.0 - 1.0;
        filter.set_transition(transition);

        let osc_out = osc.process();
        let filtered = filter.process(osc_out);
        samples.push(filtered * 0.5);
    }

    write_wav("target/test_sem_transition.wav", &samples);
    println!("Generated: target/test_sem_transition.wav");
}

#[test]
fn test_generate_delay() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);

    let mut env = Adsr::new(SAMPLE_RATE as f32);
    env.set_attack(0.01);
    env.set_decay(0.1);
    env.set_sustain(0.0);
    env.set_release(0.1);

    let mut delay = Delay::new(SAMPLE_RATE as f32);
    delay.set_delay_time(0.25); // 1/4 second
    delay.set_feedback(0.5);
    delay.set_wet(0.5);
    delay.set_dry(0.5);

    // Play notes with delay
    let notes = [220.0, 330.0, 440.0, 550.0];
    let note_interval = num_samples / notes.len();

    let mut samples = vec![0.0f32; num_samples * 2];
    let mut current_note = 0;

    for i in 0..num_samples {
        // Trigger new note
        if i % note_interval == 0 && current_note < notes.len() {
            osc.set_frequency(notes[current_note]);
            env.trigger();
            current_note += 1;
        }

        // Release after short time
        if i % note_interval == note_interval / 4 {
            env.release();
        }

        let env_val = env.process();
        let osc_out = osc.process() * env_val * 0.5;

        let (left, right) = delay.process(osc_out, osc_out);
        samples[i * 2] = left;
        samples[i * 2 + 1] = right;
    }

    write_stereo_wav("target/test_delay.wav", &samples);
    println!("Generated: target/test_delay.wav");
}

#[test]
fn test_generate_chorus() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(220.0);

    let mut chorus = Chorus::new(SAMPLE_RATE as f32);
    chorus.set_amount(0.5);
    chorus.set_dry_wet(0.7);
    chorus.set_lfo_freq(0.5);
    chorus.set_feedback(0.3);

    let mut samples = vec![0.0f32; num_samples * 2];

    for i in 0..num_samples {
        let osc_out = osc.process() * 0.5;
        let (left, right) = chorus.process(osc_out);
        samples[i * 2] = left;
        samples[i * 2 + 1] = right;
    }

    write_stereo_wav("target/test_chorus.wav", &samples);
    println!("Generated: target/test_chorus.wav");
}

#[test]
fn test_generate_lfo_modulation() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Use LFO to modulate filter cutoff
    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(110.0);

    let mut lfo = Lfo::new(SAMPLE_RATE as f32);
    lfo.set_frequency(0.5); // 0.5 Hz
    lfo.set_waveform(LfoWaveform::Sine);

    let mut filter = LadderFilter::new(SAMPLE_RATE as f32);
    filter.set_filter_type(LadderFilterType::LP4);
    filter.set_resonance(0.6);

    let mut samples = Vec::with_capacity(num_samples);

    for _ in 0..num_samples {
        // LFO modulates filter cutoff
        let lfo_val = lfo.process();
        let cutoff = 500.0 + (lfo_val * 0.5 + 0.5) * 4000.0; // 500-4500 Hz
        filter.set_cutoff(cutoff);

        let osc_out = osc.process();
        let filtered = filter.process(osc_out);
        samples.push(filtered * 0.5);
    }

    write_wav("target/test_lfo_filter.wav", &samples);
    println!("Generated: target/test_lfo_filter.wav");
}

#[test]
fn test_generate_multi_oscillator() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Test 1: Unison supersaw with different detune amounts
    for detune in [0.0f32, 0.3, 0.6, 1.0] {
        let mut osc = MultiOscillator::new(SAMPLE_RATE as f32);
        osc.set_wavetable(1); // FatSaw
        osc.set_frequency(220.0);
        osc.set_detune(detune);
        osc.randomize_phase();

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            samples.push(osc.process() * 0.5);
        }

        let filename = format!("target/test_multi_detune_{:.0}.wav", detune * 100.0);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }

    // Test 2: Stereo spread with moderate detune
    let mut osc = MultiOscillator::new(SAMPLE_RATE as f32);
    osc.set_wavetable(1); // FatSaw
    osc.set_frequency(220.0);
    osc.set_detune(0.5);
    osc.set_stereo_width(1.0);
    osc.randomize_phase();

    let mut samples = vec![0.0f32; num_samples * 2];
    for i in 0..num_samples {
        let (left, right) = osc.process_stereo();
        samples[i * 2] = left * 0.5;
        samples[i * 2 + 1] = right * 0.5;
    }

    write_stereo_wav("target/test_multi_stereo.wav", &samples);
    println!("Generated: target/test_multi_stereo.wav");

    // Test 3: Chord pad (multiple notes layered with multi osc)
    let chord_notes = [220.0, 277.18, 329.63]; // A, C#, E (A major)
    let mut samples = vec![0.0f32; num_samples * 2];

    for &freq in &chord_notes {
        let mut osc = MultiOscillator::new(SAMPLE_RATE as f32);
        osc.set_wavetable(5); // Gentle pad wavetable
        osc.set_frequency(freq);
        osc.set_detune(0.4);
        osc.set_stereo_width(0.8);
        osc.randomize_phase();

        for i in 0..num_samples {
            let (left, right) = osc.process_stereo();
            samples[i * 2] += left * 0.2;
            samples[i * 2 + 1] += right * 0.2;
        }
    }

    write_stereo_wav("target/test_multi_chord.wav", &samples);
    println!("Generated: target/test_multi_chord.wav");
}

#[test]
fn test_generate_distortion() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(220.0);

    // Test each distortion algorithm
    let algorithms = [
        (DistortionAlgorithm::Clamp, "clamp"),
        (DistortionAlgorithm::Fold, "fold"),
        (DistortionAlgorithm::Zero, "zero"),
        (DistortionAlgorithm::Sine, "sine"),
        (DistortionAlgorithm::Cube, "cube"),
    ];

    for (algo, name) in algorithms {
        let mut dist = OversamplingDistortion::new();
        dist.set_algorithm(algo);
        dist.set_threshold(0.7);
        dist.set_dry_wet(1.0);

        osc.reset();

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let input = osc.process() * 0.8;
            let output = dist.process(input);
            samples.push(output.clamp(-1.0, 1.0) * 0.5);
        }

        let filename = format!("target/test_distortion_{}.wav", name);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }

    // Test drive/threshold sweep with clamp
    let mut dist = OversamplingDistortion::new();
    dist.set_algorithm(DistortionAlgorithm::Clamp);
    dist.set_dry_wet(1.0);

    osc.reset();

    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        // Sweep threshold from low (heavy distortion) to high (clean)
        let t = i as f32 / num_samples as f32;
        dist.set_threshold(t);

        let input = osc.process() * 0.8;
        let output = dist.process(input);
        samples.push(output.clamp(-1.0, 1.0) * 0.5);
    }

    write_wav("target/test_distortion_sweep.wav", &samples);
    println!("Generated: target/test_distortion_sweep.wav");
}

#[test]
fn test_generate_bitcrusher() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);
    osc.set_frequency(220.0);

    // Test different bit depths
    for bits in [2u8, 4, 6, 8, 12] {
        let mut bc = Bitcrusher::new();
        bc.set_bits(bits);
        bc.set_edge(0.9);
        bc.set_dry_wet(1.0);

        osc.reset();

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let input = osc.process() * 0.8;
            let output = bc.process_mono(input);
            samples.push(output * 0.5);
        }

        let filename = format!("target/test_bitcrusher_{}_bits.wav", bits);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }

    // Test edge/smoothing sweep
    let mut bc = Bitcrusher::new();
    bc.set_bits(4);
    bc.set_dry_wet(1.0);

    osc.reset();

    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        // Sweep edge from soft (0) to hard (1)
        let t = i as f32 / num_samples as f32;
        bc.set_edge(t);

        let input = osc.process() * 0.8;
        let output = bc.process_mono(input);
        samples.push(output * 0.5);
    }

    write_wav("target/test_bitcrusher_edge_sweep.wav", &samples);
    println!("Generated: target/test_bitcrusher_edge_sweep.wav");
}

#[test]
fn test_generate_biquad_eq() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Use noise as input to better hear EQ effects
    let mut noise = NoiseOscillator::new(SAMPLE_RATE as f32);
    noise.set_filter_freqs(20000.0, 20.0);

    // Test peaking EQ with different frequencies
    let frequencies = [200.0, 500.0, 1000.0, 2000.0, 5000.0];

    for freq in frequencies {
        let mut eq = BiquadEQ::new(SAMPLE_RATE as f32);
        eq.set_freq(freq);
        eq.set_q(4.0);
        eq.set_gain(12.0); // +12dB boost

        noise.reset();

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let input = noise.process() * 0.3;
            let output = eq.process(input);
            samples.push(output * 0.5);
        }

        let filename = format!("target/test_eq_peak_{:.0}hz.wav", freq);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }

    // Test different band types
    let band_configs = [
        (EQBandType::LowShelf, 200.0, "lowshelf"),
        (EQBandType::HighShelf, 4000.0, "highshelf"),
        (EQBandType::LowPass, 2000.0, "lowpass"),
        (EQBandType::HighPass, 500.0, "highpass"),
    ];

    for (band_type, freq, name) in band_configs {
        let mut band = EQBand::new(SAMPLE_RATE as f32);
        band.set_band_type(band_type);
        band.set_freq(freq);
        band.set_q(1.0);
        band.set_gain(12.0);

        noise.reset();

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let input = noise.process() * 0.3;
            let output = band.process(input);
            samples.push(output * 0.5);
        }

        let filename = format!("target/test_eq_{}.wav", name);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_parametric_eq() {
    let duration_secs = 4.0;
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    // Use noise as input
    let mut noise = NoiseOscillator::new(SAMPLE_RATE as f32);
    noise.set_filter_freqs(20000.0, 20.0);

    // Test with different EQ curves
    let curves = [
        ("bass_boost", 6.0, 0.0, 0.0),
        ("mid_scoop", 0.0, -6.0, 0.0),
        ("treble_boost", 0.0, 0.0, 6.0),
        ("smiley", 4.0, -3.0, 4.0),
    ];

    for (name, low, mid, high) in curves {
        let mut eq = ParametricEQ::new(SAMPLE_RATE as f32);
        eq.set_low_gain(low);
        eq.set_mid_gain(mid);
        eq.set_high_gain(high);

        noise.reset();

        let mut samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let input = noise.process() * 0.3;
            let output = eq.process(input);
            samples.push(output * 0.4);
        }

        let filename = format!("target/test_parametric_eq_{}.wav", name);
        write_wav(&filename, &samples);
        println!("Generated: {}", filename);
    }
}

#[test]
fn test_generate_drift() {
    let duration_secs = 10.0; // Long duration to hear the slow drift
    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;

    let base_freq = 440.0;
    let drift_amount = 5.0; // +/- 5 cents of drift

    let mut osc = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc.set_waveform(Waveform::Saw);

    let mut drift = DriftGenerator::new(SAMPLE_RATE as f32);
    drift.set_drift_period(3.0); // 3 second drift cycle
    drift.set_amount(1.0);

    let mut samples = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        // Apply drift to frequency (in cents)
        let drift_val = drift.process();
        let cents_offset = drift_val * drift_amount;
        // Convert cents to frequency multiplier: 2^(cents/1200)
        let freq_mult = libm::powf(2.0, cents_offset / 1200.0);
        let freq = base_freq * freq_mult;

        osc.set_frequency(freq);
        samples.push(osc.process() * 0.5);
    }

    write_wav("target/test_drift_pitch.wav", &samples);
    println!("Generated: target/test_drift_pitch.wav");

    // Also test drift applied to filter cutoff
    let mut osc2 = AnalogOscillator::new(SAMPLE_RATE as f32);
    osc2.set_waveform(Waveform::Saw);
    osc2.set_frequency(110.0);

    let mut filter = LadderFilter::new(SAMPLE_RATE as f32);
    filter.set_filter_type(LadderFilterType::LP4);
    filter.set_resonance(0.6);

    let mut drift2 = DriftGenerator::new_with_seed(SAMPLE_RATE as f32, 54321);
    drift2.set_drift_period(4.0);
    drift2.set_amount(1.0);

    let base_cutoff = 1000.0;
    let cutoff_drift = 200.0; // +/- 200 Hz drift

    samples.clear();
    for _ in 0..num_samples {
        let drift_val = drift2.process();
        let cutoff = base_cutoff + drift_val * cutoff_drift;
        filter.set_cutoff(cutoff);

        let osc_out = osc2.process();
        let filtered = filter.process(osc_out);
        samples.push(filtered * 0.5);
    }

    write_wav("target/test_drift_filter.wav", &samples);
    println!("Generated: target/test_drift_filter.wav");
}
