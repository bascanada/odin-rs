# Testing Preset Morphing

## 🎵 Quick Start: Generate & Listen to Audio

### Option 1: Generate Audio Files (Recommended)

Generate 10 WAV files demonstrating emotional morphing:

```bash
make demo-morphing
```

**This creates files in `samples/morphing/`:**

### Morphing Files (Happy → Sad transition)
- `morph_00_pure_happy.wav` - 100% Happy (bright, fast)
- `morph_25_mostly_happy.wav` - 75% Happy, 25% Sad
- `morph_50_halfway.wav` - 50/50 blend
- `morph_75_mostly_sad.wav` - 25% Happy, 75% Sad
- `morph_100_pure_sad.wav` - 100% Sad (dark, slow)

### 2D Emotional Space Files
- `2d_happy.wav` - High valence, high arousal (joyful, energetic)
- `2d_sad.wav` - Low valence, low arousal (melancholic, slow)
- `2d_angry.wav` - Low valence, high arousal (aggressive, harsh)
- `2d_calm.wav` - High valence, low arousal (peaceful, smooth)
- `2d_neutral.wav` - Medium valence & arousal (balanced)

**On macOS:** The `morph_50_halfway.wav` file will play automatically!

### Option 2: Quick Test (No Audio Files)

Just verify the morphing works without generating files:

```bash
cd crates/odin2-core
cargo run --example preset_morph --features std
```

Or test with all presets:

```bash
make demo-all
```

This will:
- Show preset characteristics
- Display morphing parameters at different levels
- Generate a quick audio test
- Show usage examples

## 🎧 Listening to the Results

### macOS
```bash
# Play a single file
afplay morph_50_halfway.wav

# Play all morphing files in sequence
for f in morph_*.wav; do echo "Playing $f"; afplay "$f"; done

# Play all 2D emotional files
for f in 2d_*.wav; do echo "Playing $f"; afplay "$f"; done
```

### Linux
```bash
# Using aplay (ALSA)
aplay morph_50_halfway.wav

# Using paplay (PulseAudio)
paplay morph_50_halfway.wav

# Using mpv
mpv morph_50_halfway.wav
```

### Windows
```bash
# Using PowerShell
Start-Process morph_50_halfway.wav

# Or just double-click the WAV files in Explorer
```

## 🎹 What You Should Hear

### Morphing Sequence (Happy → Sad)
Listen to the files in order (00, 25, 50, 75, 100) and notice:

1. **Filter Frequency**
   - Happy: Bright (8000 Hz) - crisp, clear
   - Sad: Dark (800 Hz) - muffled, mellow

2. **Attack Time**
   - Happy: Fast (0.01s) - snappy, percussive
   - Sad: Slow (0.5s) - smooth, gradual

3. **Pitch**
   - Happy: Higher octaves
   - Sad: Lower octaves

4. **Overall Mood**
   - Happy: Energetic, uplifting
   - Halfway: Bittersweet, transitional
   - Sad: Melancholic, contemplative

### 2D Emotional Space
Compare the corner emotions:

- **Happy** vs **Sad**: Energy and brightness contrast
- **Happy** vs **Angry**: Both energetic but different valence (positive vs negative)
- **Sad** vs **Calm**: Both low energy but different valence
- **Angry** vs **Calm**: Energy contrast (high vs low)
- **Neutral**: Balanced, middle ground

## 🔬 Technical Details

Each WAV file contains:
- **Sample Rate:** 44,100 Hz
- **Channels:** Stereo
- **Duration:** ~4 seconds
- **Melody:** C major arpeggio (C-E-G-C-G-E-C)
- **Format:** 16-bit PCM

## 🎮 Using in Your Game

### Pattern 1: Simple Health-Based Morphing
```rust
// Load once
let happy = OdinPreset::create_happy();
let sad = OdinPreset::create_sad();

// Update based on player health
let health_percent = player.health / 100.0;
let emotion = 1.0 - health_percent; // 0.0 = healthy/happy, 1.0 = low/sad
let sound = happy.interpolate(&sad, emotion);
engine.load_preset(&sound);
```

### Pattern 2: 2D Emotion Space
```rust
// Update based on game state
let is_winning = player.score > enemy.score;
let combat_intensity = calculate_intensity(); // 0.0 to 1.0

let valence = if is_winning { 0.8 } else { 0.2 };
let arousal = combat_intensity;

let sound = OdinPreset::create_emotional_2d(valence, arousal);
engine.load_preset(&sound);
```

## 📊 Comparing Presets

Want to see the exact parameter differences? Run the test mode:

```bash
cd crates/odin2-core
cargo run --example preset_morph --features std
```

This shows detailed parameter values for each preset and morph level.

## 🐛 Troubleshooting

### "No audio output" or "Very quiet"
The generated audio is intentionally moderate volume. If it's too quiet:
- Check your system volume
- Try: `afplay -v 2 morph_50_halfway.wav` (2x volume on macOS)

### "Files not generated"
Make sure you include the `--generate-audio` flag:
```bash
cargo run --example preset_morph --features std -- --generate-audio
```

### "Can't hear the difference"
Try listening to the extremes first:
1. `morph_00_pure_happy.wav` - Bright and fast
2. `morph_100_pure_sad.wav` - Dark and slow

The difference should be very noticeable!

## 🎓 Understanding the Presets

### Happy (Bright, Energetic)
- **Use for:** Victory, joy, success
- **Filter:** 8000 Hz (very bright)
- **Attack:** 0.01s (instant)
- **Oscillators:** Higher octaves
- **Interval:** Perfect fifth (cheerful)

### Sad (Dark, Mellow)
- **Use for:** Defeat, loss, sadness
- **Filter:** 800 Hz (very dark)
- **Attack:** 0.5s (slow fade-in)
- **Oscillators:** Lower octave
- **Interval:** Minor third (melancholic)

### Angry (Harsh, Aggressive)
- **Use for:** Combat, danger, tension
- **Filter:** 3000 Hz + high resonance (harsh)
- **Attack:** 0.001s (percussive)
- **Oscillators:** Tritone (dissonant)

### Calm (Smooth, Balanced)
- **Use for:** Exploration, rest, peace
- **Filter:** 2500 Hz (mid-range)
- **Attack:** 0.2s (gentle)
- **Oscillators:** Major third (pleasant)

## 📚 Next Steps

1. **Listen to all files** to understand the emotional range
2. **Experiment** with different morph levels in your game
3. **Create custom presets** following the existing patterns
4. **Integrate** into your procedural music system

## 💡 Pro Tips

- Use smooth morphing (`interpolate_smooth`) for gradual mood changes
- Cache preset interpolations if using the same values repeatedly
- Update morphing only when emotion changes significantly (> 0.05 delta)
- Combine morphing with different melodies for more variety

## 🎵 Have Fun!

The generated files demonstrate the full range of emotional expression possible with procedural preset morphing. Listen, experiment, and create adaptive music for your game!
