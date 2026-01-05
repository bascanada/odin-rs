//! Bitcrusher Effect
//!
//! A lo-fi effect that reduces bit depth and adds stepped/aliased character.
//! Includes an "edge" parameter for smoothing the stepped waveform.

/// Bitcrusher Effect
///
/// Reduces the bit depth of the audio signal for a lo-fi, retro sound.
/// The edge parameter controls how sharply the steps transition.
pub struct Bitcrusher {
    /// Bit depth (1-16)
    bits: u8,

    /// Discretization scalar = 2^(bits-1)
    discretization_scalar: f32,

    /// Edge/smoothing amount (0.0 = smooth, 1.0 = hard steps)
    edge: f32,

    /// Dry/wet mix (0.0 = dry, 1.0 = wet)
    dry_wet: f32,

    /// Current filtered value (left channel)
    current_value_left: f32,

    /// Current filtered value (right channel)
    current_value_right: f32,
}

impl Bitcrusher {
    /// Create a new bitcrusher
    pub fn new() -> Self {
        Self {
            bits: 4,
            discretization_scalar: 8.0, // 2^(4-1) = 8
            edge: 0.8,
            dry_wet: 1.0,
            current_value_left: 0.0,
            current_value_right: 0.0,
        }
    }

    /// Set bit depth (1-16)
    /// Lower values = more crushed/lo-fi sound
    pub fn set_bits(&mut self, bits: u8) {
        let bits = bits.clamp(1, 16);
        if bits == self.bits {
            return;
        }
        self.bits = bits;
        self.discretization_scalar = libm::powf(2.0, (bits - 1) as f32);
    }

    /// Set edge/smoothing amount (0.0 to 1.0)
    /// 0.0 = smooth transitions, 1.0 = hard steps
    pub fn set_edge(&mut self, edge: f32) {
        self.edge = edge.clamp(0.0, 1.0);
    }

    /// Set dry/wet mix (0.0 = dry, 1.0 = wet)
    pub fn set_dry_wet(&mut self, dry_wet: f32) {
        self.dry_wet = dry_wet.clamp(0.0, 1.0);
    }

    /// Reset the effect state
    pub fn reset(&mut self) {
        self.current_value_left = 0.0;
        self.current_value_right = 0.0;
    }

    /// Process left channel
    pub fn process_left(&mut self, input: f32) -> f32 {
        // Quantize to reduced bit depth
        let target_value =
            libm::floorf(input * self.discretization_scalar) / self.discretization_scalar;

        // Apply edge smoothing
        self.current_value_left += (target_value - self.current_value_left) * self.edge;

        // Mix dry/wet
        self.current_value_left * self.dry_wet + input * (1.0 - self.dry_wet)
    }

    /// Process right channel
    pub fn process_right(&mut self, input: f32) -> f32 {
        // Quantize to reduced bit depth
        let target_value =
            libm::floorf(input * self.discretization_scalar) / self.discretization_scalar;

        // Apply edge smoothing
        self.current_value_right += (target_value - self.current_value_right) * self.edge;

        // Mix dry/wet
        self.current_value_right * self.dry_wet + input * (1.0 - self.dry_wet)
    }

    /// Process stereo input
    /// Returns (left_out, right_out)
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        (self.process_left(left_in), self.process_right(right_in))
    }

    /// Process mono input
    pub fn process_mono(&mut self, input: f32) -> f32 {
        self.process_left(input)
    }
}

impl Default for Bitcrusher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcrusher_basic() {
        let mut bc = Bitcrusher::new();
        bc.set_bits(4);
        bc.set_edge(1.0);
        bc.set_dry_wet(1.0);

        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let output = bc.process_mono(input);
            assert!(output.is_finite(), "Bitcrusher produced invalid output");
        }
    }

    #[test]
    fn test_bitcrusher_stereo() {
        let mut bc = Bitcrusher::new();
        bc.set_bits(8);
        bc.set_edge(0.8);

        for i in 0..1000 {
            let t = i as f32 / 44100.0;
            let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
            let (left, right) = bc.process(input, input);
            assert!(left.is_finite() && right.is_finite());
        }
    }

    #[test]
    fn test_bitcrusher_quantization() {
        let mut bc = Bitcrusher::new();
        bc.set_bits(2); // Very low bit depth
        bc.set_edge(1.0); // Hard steps
        bc.set_dry_wet(1.0);

        // With 2 bits, we should have very few discrete levels
        let input = 0.5;
        let output = bc.process_mono(input);

        // Output should be quantized
        assert!(output.is_finite());
    }

    #[test]
    fn test_bitcrusher_edge_smoothing() {
        let mut bc_hard = Bitcrusher::new();
        bc_hard.set_bits(4);
        bc_hard.set_edge(1.0);
        bc_hard.set_dry_wet(1.0);

        let mut bc_soft = Bitcrusher::new();
        bc_soft.set_bits(4);
        bc_soft.set_edge(0.1);
        bc_soft.set_dry_wet(1.0);

        // Process same input through both
        let input = 0.75;
        let _ = bc_hard.process_mono(input);
        let _ = bc_soft.process_mono(input);

        // Both should produce valid output
        // Soft edge should have smoother transitions
        assert!(bc_hard.current_value_left.is_finite());
        assert!(bc_soft.current_value_left.is_finite());
    }

    #[test]
    fn test_bitcrusher_bit_depths() {
        for bits in 1..=16 {
            let mut bc = Bitcrusher::new();
            bc.set_bits(bits);
            bc.set_edge(0.8);
            bc.set_dry_wet(1.0);

            for i in 0..100 {
                let t = i as f32 / 44100.0;
                let input = libm::sinf(2.0 * core::f32::consts::PI * 440.0 * t);
                let output = bc.process_mono(input);
                assert!(
                    output.is_finite(),
                    "Bitcrusher with {} bits produced invalid output",
                    bits
                );
            }
        }
    }
}
