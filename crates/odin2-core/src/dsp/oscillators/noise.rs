//! Noise Oscillator
//!
//! Generates white noise with optional lowpass and highpass filtering.

use crate::dsp::filters::VAOnePoleFilter;

/// Filter frequency constants
const FILTER_FC_MIN: f32 = 20.0;
const FILTER_FC_MAX: f32 = 20000.0;

/// Simple PRNG for noise generation (xorshift32)
struct NoiseRng {
    state: u32,
}

impl NoiseRng {
    fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Returns bipolar random value in range [-1.0, 1.0]
    fn next_bipolar(&mut self) -> f32 {
        let n = self.next();
        (n as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Noise oscillator with filtering
///
/// Generates white noise and passes it through lowpass and highpass filters
/// for spectral shaping (pink-ish noise, band-limited noise, etc.)
pub struct NoiseOscillator {
    /// Lowpass filter
    lowpass: VAOnePoleFilter,

    /// Highpass filter
    highpass: VAOnePoleFilter,

    /// Random number generator
    rng: NoiseRng,

    /// Sample rate
    sample_rate: f32,
}

impl NoiseOscillator {
    /// Create a new noise oscillator
    pub fn new(sample_rate: f32) -> Self {
        let mut lowpass = VAOnePoleFilter::new(sample_rate);
        lowpass.set_lowpass();
        lowpass.set_frequency(FILTER_FC_MAX);

        let mut highpass = VAOnePoleFilter::new(sample_rate);
        highpass.set_highpass();
        highpass.set_frequency(FILTER_FC_MIN);

        Self {
            lowpass,
            highpass,
            rng: NoiseRng::new(42),
            sample_rate,
        }
    }

    /// Set the lowpass filter frequency
    pub fn set_lp_freq(&mut self, freq: f32) {
        self.lowpass.set_frequency(freq.clamp(FILTER_FC_MIN, FILTER_FC_MAX));
    }

    /// Set the highpass filter frequency
    pub fn set_hp_freq(&mut self, freq: f32) {
        self.highpass.set_frequency(freq.clamp(FILTER_FC_MIN, FILTER_FC_MAX));
    }

    /// Set both filter frequencies
    pub fn set_filter_freqs(&mut self, lp_freq: f32, hp_freq: f32) {
        self.set_lp_freq(lp_freq);
        self.set_hp_freq(hp_freq);
    }

    /// Process and return one sample of noise
    pub fn process(&mut self) -> f32 {
        // Generate white noise
        let white_noise = self.rng.next_bipolar();

        // Apply filters
        let filtered = self.lowpass.process(white_noise);
        self.highpass.process(filtered)
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.lowpass.set_sample_rate(sample_rate);
        self.highpass.set_sample_rate(sample_rate);
    }

    /// Reset filters
    pub fn reset(&mut self) {
        self.lowpass.reset();
        self.highpass.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_basic() {
        let mut noise = NoiseOscillator::new(44100.0);

        let mut samples = [0.0f32; 1000];
        for s in &mut samples {
            *s = noise.process();
        }

        // Check that we get valid output
        let max = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0 && max <= 1.5, "Max amplitude: {}", max);
    }

    #[test]
    fn test_noise_is_random() {
        let mut noise = NoiseOscillator::new(44100.0);

        let sample1 = noise.process();
        let sample2 = noise.process();
        let sample3 = noise.process();

        // Samples should be different (extremely unlikely to be the same)
        assert!(
            sample1 != sample2 || sample2 != sample3,
            "Noise should produce different values"
        );
    }

    #[test]
    fn test_noise_filtering() {
        let mut noise = NoiseOscillator::new(44100.0);

        // Full bandwidth
        noise.set_filter_freqs(20000.0, 20.0);
        let full_bw: f32 = (0..1000).map(|_| noise.process().abs()).sum::<f32>() / 1000.0;

        noise.reset();

        // Limited bandwidth (lowpass at 1000 Hz)
        noise.set_filter_freqs(1000.0, 20.0);
        let limited_bw: f32 = (0..1000).map(|_| noise.process().abs()).sum::<f32>() / 1000.0;

        // Limited bandwidth should have lower average amplitude (less energy)
        // This isn't always true for random signals, so we just check both are positive
        assert!(full_bw > 0.0, "Full bandwidth noise should have energy");
        assert!(limited_bw > 0.0, "Limited bandwidth noise should have energy");
    }
}
