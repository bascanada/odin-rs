//! DC Blocking Filter
//!
//! A simple highpass filter to remove DC offset from signals.
//! Based on Julius O. Smith's design from CCRMA.

/// DC Blocking Filter
///
/// Removes DC offset using a first-order highpass filter.
/// y[n] = x[n] - x[n-1] + R * y[n-1]
/// where R is typically 0.995-0.999
#[derive(Debug, Clone)]
pub struct DCBlockingFilter {
    /// Previous input sample
    xm1: f32,
    /// Previous output sample
    ym1: f32,
    /// R coefficient (controls cutoff frequency)
    r: f32,
}

impl DCBlockingFilter {
    /// Create a new DC blocking filter
    pub fn new() -> Self {
        Self {
            xm1: 0.0,
            ym1: 0.0,
            r: 0.995,
        }
    }

    /// Set sample rate (adjusts R coefficient accordingly)
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        // Higher sample rates need higher R to maintain similar cutoff frequency
        if sample_rate > 120000.0 {
            self.r = 0.997;
        } else if sample_rate > 90000.0 {
            self.r = 0.9965;
        } else {
            self.r = 0.995;
        }
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.xm1 = 0.0;
        self.ym1 = 0.0;
    }

    /// Process one sample
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = input - self.xm1 + self.r * self.ym1;
        self.xm1 = input;
        self.ym1 = output;
        output
    }
}

impl Default for DCBlockingFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_blocker_removes_dc() {
        let mut filter = DCBlockingFilter::new();

        // Feed constant DC signal
        let mut output = 0.0;
        for _ in 0..10000 {
            output = filter.process(1.0);
        }

        // DC should be blocked (output near 0)
        assert!(
            output.abs() < 0.01,
            "DC blocker should remove DC offset: {}",
            output
        );
    }

    #[test]
    fn test_dc_blocker_passes_ac() {
        let mut filter = DCBlockingFilter::new();

        // Feed a square wave
        let mut energy = 0.0f32;
        for i in 0..10000 {
            let input = if i % 100 < 50 { 1.0 } else { -1.0 };
            let output = filter.process(input);
            // Skip first few samples for settling
            if i > 100 {
                energy += output.abs();
            }
        }

        // AC signal should pass through
        assert!(
            energy > 100.0,
            "DC blocker should pass AC signal: {}",
            energy
        );
    }
}
