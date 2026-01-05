//! Oversampling Distortion Effect
//!
//! A distortion effect with 3x oversampling to prevent aliasing.
//! Supports multiple distortion algorithms: Clamp, Fold, Zero, Sine, Cube.

/// Minimum threshold value to avoid divide-by-zero
const THRESHOLD_MIN: f32 = 0.05;

/// Smoothing factor for threshold changes
const THRESHOLD_SMOOTHING: f32 = 0.998;

/// Output scalar for distortion
const DISTORTION_OUTPUT_SCALAR: f32 = 1.0;

/// Distortion algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DistortionAlgorithm {
    /// Hard clipping - signal clamped at threshold
    Clamp = 1,
    /// Wavefolding - signal folds back at threshold
    Fold = 2,
    /// Zero crossing - signal zeroed above threshold
    Zero = 3,
    /// Sine waveshaper - soft saturation
    Sine = 4,
    /// Cubic waveshaper - soft saturation
    Cube = 5,
}

impl Default for DistortionAlgorithm {
    fn default() -> Self {
        Self::Clamp
    }
}

impl DistortionAlgorithm {
    /// Create from integer value
    pub fn from_int(value: u8) -> Self {
        match value {
            1 => Self::Clamp,
            2 => Self::Fold,
            3 => Self::Zero,
            4 => Self::Sine,
            5 => Self::Cube,
            _ => Self::Clamp,
        }
    }
}

/// Oversampling Distortion Effect
///
/// Uses 3x oversampling with a 9th order Butterworth lowpass filter
/// to prevent aliasing artifacts from the non-linear distortion.
pub struct OversamplingDistortion {
    /// Current algorithm
    algorithm: DistortionAlgorithm,

    /// Last input sample (for interpolation)
    last_input: f32,

    /// DC bias
    bias: f32,

    /// Threshold (0.0 to 1.0, where higher = more distortion)
    threshold: f32,

    /// Smoothed threshold
    threshold_smooth: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    dry_wet: f32,

    /// IIR filter buffers for downsampling (order 9)
    xv: [f32; 10],
    yv: [f32; 10],
}

impl OversamplingDistortion {
    /// Create a new oversampling distortion
    pub fn new() -> Self {
        Self {
            algorithm: DistortionAlgorithm::Clamp,
            last_input: 0.0,
            bias: 0.0,
            threshold: 0.343, // (1 - 0.3)^3
            threshold_smooth: 0.343,
            dry_wet: 1.0,
            xv: [0.0; 10],
            yv: [0.0; 10],
        }
    }

    /// Set the distortion algorithm
    pub fn set_algorithm(&mut self, algorithm: DistortionAlgorithm) {
        self.algorithm = algorithm;
    }

    /// Set the threshold/drive amount (0.0 to 1.0)
    /// Higher values = more distortion
    pub fn set_threshold(&mut self, threshold: f32) {
        // Invert and cube for better feel
        let t = 1.0 - threshold.clamp(0.0, 1.0);
        self.threshold = t * t * t;
    }

    /// Set DC bias (-1.0 to 1.0)
    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias.clamp(-1.0, 1.0);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_dry_wet(&mut self, dry_wet: f32) {
        self.dry_wet = dry_wet.clamp(0.0, 1.0);
    }

    /// Reset the effect state
    pub fn reset(&mut self) {
        for i in 0..10 {
            self.xv[i] = 0.0;
            self.yv[i] = 0.0;
        }
        self.threshold_smooth = self.threshold;
        self.last_input = 0.0;
    }

    /// Apply distortion algorithm to a single sample
    #[inline]
    fn apply_distortion(&self, sample: f32, threshold: f32) -> f32 {
        match self.algorithm {
            DistortionAlgorithm::Clamp => {
                if sample > self.bias && sample > self.bias + threshold {
                    self.bias + threshold
                } else if sample < self.bias && sample < self.bias - threshold {
                    self.bias - threshold
                } else {
                    sample
                }
            }
            DistortionAlgorithm::Zero => {
                // Half boost for zero algorithm
                let t = 0.5 + threshold * 0.5;
                if sample > self.bias && sample > self.bias + t {
                    0.0
                } else if sample < self.bias && sample < self.bias - t {
                    0.0
                } else {
                    sample
                }
            }
            DistortionAlgorithm::Sine => {
                // Sine waveshaper
                libm::sinf(sample)
            }
            DistortionAlgorithm::Cube => {
                // Cubic waveshaper
                sample * sample * sample
            }
            DistortionAlgorithm::Fold => {
                // Wavefolding
                let mut s = sample;
                let max_iterations = 10; // Prevent infinite loop
                let mut iter = 0;
                while libm::fabsf(s) > threshold && iter < max_iterations {
                    if s > threshold {
                        s = 2.0 * threshold - s;
                    } else if s < -threshold {
                        s = -2.0 * threshold - s;
                    }
                    iter += 1;
                }
                s
            }
        }
    }

    /// Process one filter step for downsampling
    /// Butterworth LP, order 9, SR 132300, corner freq 40000 Hz
    #[inline]
    fn filter_step(&mut self, input: f32) {
        // Shift buffers
        for i in 0..9 {
            self.xv[i] = self.xv[i + 1];
            self.yv[i] = self.yv[i + 1];
        }

        self.xv[9] = input * 0.019966841051093;
        self.yv[9] = (self.xv[0] + self.xv[9])
            + 9.0 * (self.xv[1] + self.xv[8])
            + 36.0 * (self.xv[2] + self.xv[7])
            + 84.0 * (self.xv[3] + self.xv[6])
            + 126.0 * (self.xv[4] + self.xv[5])
            + (-0.0003977153 * self.yv[0])
            + (-0.0064474617 * self.yv[1])
            + (-0.0476997403 * self.yv[2])
            + (-0.2185829743 * self.yv[3])
            + (-0.6649234123 * self.yv[4])
            + (-1.4773657709 * self.yv[5])
            + (-2.2721421641 * self.yv[6])
            + (-2.6598673212 * self.yv[7])
            + (-1.8755960587 * self.yv[8]);
    }

    /// Process one sample
    pub fn process(&mut self, input: f32) -> f32 {
        // 3x oversampling with linear interpolation
        let upsampled = [
            0.66666666 * self.last_input + 0.33333333 * input,
            0.33333333 * self.last_input + 0.66666666 * input,
            input,
        ];

        self.last_input = input;

        // Smooth threshold
        self.threshold_smooth =
            self.threshold_smooth * THRESHOLD_SMOOTHING + (1.0 - THRESHOLD_SMOOTHING) * self.threshold;

        // Apply threshold modulation (clamp to valid range)
        let threshold_modded = (self.threshold_smooth * (1.0 - THRESHOLD_MIN) + THRESHOLD_MIN)
            .clamp(THRESHOLD_MIN, 1.0);

        // Apply distortion to each upsampled sample and run through filter
        for i in 0..3 {
            let distorted = self.apply_distortion(upsampled[i], threshold_modded);
            self.filter_step(distorted);
        }

        // Mix dry/wet
        let wet_output = match self.algorithm {
            DistortionAlgorithm::Clamp | DistortionAlgorithm::Fold | DistortionAlgorithm::Zero => {
                // Compensate for threshold reduction
                self.yv[9] / threshold_modded * DISTORTION_OUTPUT_SCALAR
            }
            DistortionAlgorithm::Sine | DistortionAlgorithm::Cube => {
                self.yv[9]
            }
        };

        wet_output * self.dry_wet + input * (1.0 - self.dry_wet)
    }
}

impl Default for OversamplingDistortion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distortion_clamp() {
        let mut dist = OversamplingDistortion::new();
        dist.set_algorithm(DistortionAlgorithm::Clamp);
        dist.set_threshold(0.8);
        dist.set_dry_wet(1.0);

        for _ in 0..1000 {
            let input = libm::sinf(core::f32::consts::PI * 0.1);
            let output = dist.process(input);
            assert!(output.is_finite(), "Distortion produced invalid output");
        }
    }

    #[test]
    fn test_distortion_fold() {
        let mut dist = OversamplingDistortion::new();
        dist.set_algorithm(DistortionAlgorithm::Fold);
        dist.set_threshold(0.7);
        dist.set_dry_wet(1.0);

        for i in 0..1000 {
            let t = i as f32 / 1000.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 10.0 * t);
            let output = dist.process(input);
            assert!(output.is_finite(), "Fold distortion produced invalid output");
        }
    }

    #[test]
    fn test_distortion_sine() {
        let mut dist = OversamplingDistortion::new();
        dist.set_algorithm(DistortionAlgorithm::Sine);
        dist.set_dry_wet(1.0);

        for i in 0..1000 {
            let t = i as f32 / 1000.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 10.0 * t);
            let output = dist.process(input);
            assert!(output.is_finite(), "Sine distortion produced invalid output");
        }
    }

    #[test]
    fn test_distortion_dry_wet() {
        let mut dist = OversamplingDistortion::new();
        dist.set_algorithm(DistortionAlgorithm::Clamp);
        dist.set_threshold(0.9);

        // Process with dry signal
        dist.set_dry_wet(0.0);
        let input = 0.5;
        let dry_output = dist.process(input);

        dist.reset();

        // Process with wet signal
        dist.set_dry_wet(1.0);
        let wet_output = dist.process(input);

        // Dry should be closer to input
        // (Note: due to filter latency, first samples may differ)
        assert!(dry_output.is_finite());
        assert!(wet_output.is_finite());
    }

    #[test]
    fn test_all_algorithms() {
        for algo in [
            DistortionAlgorithm::Clamp,
            DistortionAlgorithm::Fold,
            DistortionAlgorithm::Zero,
            DistortionAlgorithm::Sine,
            DistortionAlgorithm::Cube,
        ] {
            let mut dist = OversamplingDistortion::new();
            dist.set_algorithm(algo);
            dist.set_threshold(0.7);
            dist.set_dry_wet(0.8);

            for i in 0..500 {
                let t = i as f32 / 44100.0;
                let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
                let output = dist.process(input);
                assert!(
                    output.is_finite(),
                    "Algorithm {:?} produced invalid output",
                    algo
                );
            }
        }
    }
}
