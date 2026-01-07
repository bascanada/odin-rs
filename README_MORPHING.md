# 🎵 Preset Morphing & Emotional Sound Design

## What is this?

odin-rs now includes **procedural emotional presets** and **smooth morphing** for creating adaptive game music. No external preset files needed - everything is built-in!

## Quick Demo

```bash
# Generate 10 audio files demonstrating emotional morphing
make demo-morphing

# Or generate all built-in presets
make demo-all

# Or run all demos
make demos

# Listen to the results (macOS will auto-play one file)
# All files are saved in samples/ subdirectories
```

## Generated Files

You'll get **10 WAV files** (~4 seconds each):

### 1D Morphing (Happy → Sad)
- `morph_00_pure_happy.wav` ⚡ Bright, fast, energetic
- `morph_25_mostly_happy.wav` 
- `morph_50_halfway.wav` 🎭 Bittersweet blend
- `morph_75_mostly_sad.wav`
- `morph_100_pure_sad.wav` 😔 Dark, slow, mellow

### 2D Emotional Space
- `2d_happy.wav` 😊 High valence, high arousal
- `2d_sad.wav` 😢 Low valence, low arousal
- `2d_angry.wav` 😠 Low valence, high arousal
- `2d_calm.wav` 😌 High valence, low arousal
- `2d_neutral.wav` 😐 Balanced

## Use in Your Game

### Simple Example

```rust
use odin2_core::preset::OdinPreset;
use odin2_core::engine::{OdinEngine, SynthEngine};

// Setup (once)
let mut engine = OdinEngine::new(44100.0);
let happy = OdinPreset::create_happy();
let sad = OdinPreset::create_sad();

// Game loop
let player_health = 0.3; // 30% health
let emotion = 1.0 - player_health; // 0.7 = 70% sad
let sound = happy.interpolate(&sad, emotion);
engine.load_preset(&sound);

// Play
engine.note_on(60, 100); // C4
```

### 2D Emotional Space

```rust
// Based on game state
let valence = if player_winning { 0.8 } else { 0.2 };
let arousal = combat_intensity; // 0.0 to 1.0

let sound = OdinPreset::create_emotional_2d(valence, arousal);
engine.load_preset(&sound);
```

## Documentation

- **[TESTING_MORPHING.md](TESTING_MORPHING.md)** - How to test and listen to audio
- **[PRESET_MORPHING.md](PRESET_MORPHING.md)** - Complete API reference and examples

## Features

✅ **4 Procedural Emotional Presets** (Happy, Sad, Angry, Calm)  
✅ **1D Morphing** - Smooth transitions between two emotions  
✅ **2D Emotional Space** - Valence × Arousal model  
✅ **Real-time Performance** - < 5µs per interpolation  
✅ **No External Files** - Everything built-in  
✅ **Ready for Games** - Easy integration

## Technical Details

- **Sample Rate:** 44,100 Hz
- **Tests:** 131 passing
- **Performance:** Real-time capable
- **API:** Simple and intuitive

## What's Next?

After generating the audio files:

1. **Listen** to understand the emotional range
2. **Read** [PRESET_MORPHING.md](PRESET_MORPHING.md) for integration examples
3. **Integrate** into your game's audio system
4. **Experiment** with custom presets

Enjoy creating adaptive, emotional music! 🎮🎵
