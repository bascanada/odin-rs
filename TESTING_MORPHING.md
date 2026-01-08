# Testing Scatter Morphing

## Running Tests

To verify the core interpolation logic:

```bash
# Run unit tests
cargo test --package odin2-core interpolation
```

## Generating Audio Demos

To generate WAV files demonstrating the morphing system:

```bash
# Using the dedicated demo command
make demo-morphing
```

Or via cargo directly:

```bash
cargo run --example preset_morph --features std -- --generate-audio
```

This will output WAV files to `samples/morphing/`.

### Output Interpretation

- **morph_center.wav**: A blend of all source presets (should sound balanced).
- **morph_top_left.wav**: Should sound identical to the top-left source preset.
- **morph_top_right.wav**: Should sound identical to the top-right source preset.
- etc.

## Performance Benchmark

To ensure real-time performance inside a game loop, run the benchmarks:

```bash
cargo bench --package odin2-core
```

Target performance is under 10µs per morph operation.
