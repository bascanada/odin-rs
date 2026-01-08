# Odin 2 Preset Morphing API

## Overview

The Preset Morphing API allows for real-time interpolation between multiple `OdinPreset` instances. This is useful for:
- Creating dynamic, evolving sounds.
- Implementing "X/Y Pad" interfaces.
- Procedural audio generation based on game state.

## Core Concepts

### 2D Scatter Morphing

The system moves away from simple linear interpolation (A to B) and instead uses a **geometric** approach. You define any number of "source" presets, each with an (X, Y) coordinate. You can then request a blended preset at any target (X, Y) location.

The blending algorithm uses **Inverse Distance Weighting (IDW)**. Presets closer to the target point have more influence.

## API Reference

### `OdinPreset::morph_2d`

Interpolates between multiple presets based on 2D coordinates.

```rust
pub fn morph_2d(sources: &[(OdinPreset, f32, f32)], x: f32, y: f32) -> Self
```

- **sources**: A slice of tuples, where each tuple contains:
  - The source `OdinPreset`.
  - The X coordinate (f32).
  - The Y coordinate (f32).
- **x**: The target X coordinate.
- **y**: The target Y coordinate.
- **Returns**: A new `OdinPreset` with interpolated parameters.

### `OdinPreset::interpolate`

Linearly interpolates between two presets.

```rust
pub fn interpolate(&self, other: &Self, t: f32) -> Self
```

- **other**: The target preset to morph towards.
- **t**: Interpolation factor (0.0 = self, 1.0 = other).

## Examples

### Basic 4-Corner Morphing

This setup mimics a standard X/Y controller where each corner corresponds to a different sound.

```rust
// 1. Load your presets
let p_top_left = OdinPreset::load("A.odin")?;
let p_top_right = OdinPreset::load("B.odin")?;
let p_btm_left = OdinPreset::load("C.odin")?;
let p_btm_right = OdinPreset::load("D.odin")?;

// 2. Define geometry
let sources = vec![
    (p_top_left, -1.0, 1.0),
    (p_top_right, 1.0, 1.0),
    (p_btm_left, -1.0, -1.0),
    (p_btm_right, 1.0, -1.0),
];

// 3. Morph at input coordinates
// Mouse input, gamepad stick, etc.
let input_x = 0.5; 
let input_y = -0.2;

let blended = OdinPreset::morph_2d(&sources, input_x, input_y);
```
