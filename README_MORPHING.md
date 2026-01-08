# 🎵 2D Scatter Morphing System

## What is this?

odin-rs includes a **generic 2D scatter morphing system**. This allows you to place any number of presets at arbitrary coordinates on a 2D plane (X, Y) and smoothly interpolate between them based on proximity.

## Quick Demo

```bash
# Generate morphing examples
make demo-morphing
```

## How It Works

1. **Define Source Presets**: Load existing presets or create them programmatically.
2. **Assign Coordinates**: Place each preset at a specific (X, Y) location (e.g., corners of a square).
3. **Interpolate**: Request a new preset at any target (X, Y) coordinate. The system calculates a weighted blend of all source presets using Inverse Distance Weighting (IDW).

## Use in Your Game

### 2D Scatter Example

```rust
use odin2_core::preset::OdinPreset;
use odin2_core::engine::{OdinEngine, SynthEngine};

// Setup
let mut engine = OdinEngine::new(44100.0);

// Load or create source presets
let p1 = OdinPreset::load("presets/Bass.odin").unwrap();
let p2 = OdinPreset::load("presets/Pad.odin").unwrap();
let p3 = OdinPreset::load("presets/Lead.odin").unwrap();
let p4 = OdinPreset::load("presets/FX.odin").unwrap();

// Define mapping (Preset, X, Y)
let sources = vec![
    (p1, -1.0, 1.0),   // Top-Left
    (p2, 1.0, 1.0),    // Top-Right
    (p3, -1.0, -1.0),  // Bottom-Left
    (p4, 1.0, -1.0),   // Bottom-Right
];

// Game loop
let tension = get_game_tension(); // -1.0 to 1.0
let action = get_game_action();   // -1.0 to 1.0

// Get blended preset
let blended = OdinPreset::morph_2d(&sources, tension, action);

// Apply to engine
engine.load_preset(&blended);
engine.note_on(60, 100);
```

## Features

✅ **Generic 2D Morphing** - Interpolate between N presets  
✅ **Inverse Distance Weighting** - Smooth, distance-based blending  
✅ **Real-time Performance** - Optimized for game loops  
✅ **Flexible Coordinate System** - Use any range (typically -1.0 to 1.0)  

## Documentation

- **[TESTING_MORPHING.md](TESTING_MORPHING.md)** - How to test the system
- **[PRESET_MORPHING.md](PRESET_MORPHING.md)** - Detailed API reference
