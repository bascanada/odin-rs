# Odin 2 Rust Port - Modules Restants

Ce document liste les modules C++ d'Odin 2 qui restent à porter en Rust.

## Statut Global

- **Oscillateurs**: 11/11 (100%) ✅
- **Filtres**: 10/10 (100%) ✅
- **Effets**: 12/13 (92%) - ZitaReverb ajouté

**Tests**: 145 total (118 unit + 27 audio generation)

---

## Oscillateurs Manquants

### 1. MultiOscillator
- **Fichier C++**: `Source/audio/Oscillators/MultiOscillator.h/cpp`
- **Description**: Oscillateur multi-voix avec unison, detune et spread stéréo
- **Priorité**: HAUTE (essentiel pour pads/leads)
- **Statut**: [x] Implémenté (`multi.rs`)
- **Test WAV**: [x] `test_multi_*.wav`

### 2. WavetableOsc2D
- **Fichier C++**: `Source/audio/Oscillators/WavetableOsc2D.h/cpp`
- **Description**: Wavetable 2D avec morphing entre wavetables
- **Priorité**: MOYENNE
- **Statut**: [x] Implémenté (`wavetable2d.rs`)
- **Test WAV**: [x] `test_wavetable2d_*.wav`

### 3. DriftGenerator
- **Fichier C++**: `Source/audio/Oscillators/DriftGenerator.h/cpp`
- **Description**: Génère du drift analogique lent pour les oscillateurs
- **Priorité**: BASSE (amélioration sonore)
- **Statut**: [x] Implémenté (`drift.rs`)
- **Test WAV**: [x] `test_drift_*.wav`

---

## Filtres Manquants

### 4. BiquadEQ
- **Fichier C++**: `Source/audio/Filters/BiquadEQ.h/cpp`
- **Description**: EQ paramétrique avec différents types de bandes
- **Priorité**: MOYENNE
- **Statut**: [x] Implémenté (`eq.rs` - BiquadEQ + EQBand)
- **Test WAV**: [x] `test_eq_*.wav`

---

## Effets Manquants

### 5. Bitcrusher
- **Fichier C++**: `Source/audio/FX/Bitcrusher.h/cpp`
- **Description**: Réduction de bits et sample rate pour effet lo-fi
- **Priorité**: MOYENNE
- **Statut**: [x] Implémenté (`bitcrusher.rs`)
- **Test WAV**: [x] `test_bitcrusher_*.wav`

### 6. OversamplingDistortion
- **Fichier C++**: `Source/audio/FX/OversamplingDistortion.h/cpp`
- **Description**: Distortion avec oversampling pour éviter l'aliasing
- **Priorité**: HAUTE (effet essentiel)
- **Statut**: [x] Implémenté (`distortion.rs`)
- **Test WAV**: [x] `test_distortion_*.wav`

### 7. ParametricEQ
- **Fichier C++**: `Source/audio/FX/ParametricEQ.h/cpp`
- **Description**: EQ paramétrique master avec plusieurs bandes
- **Priorité**: MOYENNE
- **Statut**: [x] Implémenté (`parametric_eq.rs`)
- **Test WAV**: [x] `test_parametric_eq_*.wav`

### 8. ZitaReverb
- **Fichier C++**: `Source/audio/FX/ZitaReverb.h/cpp`
- **Description**: Réverbération algorithmique basée sur Zita-Rev1
- **Priorité**: HAUTE (effet essentiel)
- **Statut**: [x] Implémenté (`reverb.rs`)
- **Test WAV**: [x] `test_reverb_*.wav`

### 9. SurgeReverb (optionnel)
- **Fichier C++**: `Source/audio/FX/SurgeReverb.h/cpp`
- **Description**: Réverbération alternative du synthétiseur Surge
- **Priorité**: BASSE (alternative à Zita)
- **Statut**: [ ] Optionnel - non implémenté
- **Test WAV**: [ ] N/A

### 10. FeedbackDelayNetwork
- **Fichier C++**: `Source/audio/FX/FeedbackDelayNetwork.h/cpp`
- **Description**: Réseau de délais pour reverb (composant interne)
- **Priorité**: HAUTE (requis par reverbs)
- **Statut**: [x] Intégré dans ZitaReverb (`reverb.rs`)
- **Test WAV**: [x] N/A (composant interne)

---

## Ordre d'Implémentation Recommandé

1. **MultiOscillator** - Critique pour le son
2. **OversamplingDistortion** - Effet très utilisé
3. **FeedbackDelayNetwork** - Requis pour reverb
4. **ZitaReverb** - Reverb principale
5. **Bitcrusher** - Effet lo-fi populaire
6. **BiquadEQ** - Utile pour shaping
7. **ParametricEQ** - EQ master
8. **WavetableOsc2D** - Alternative à Vector
9. **DriftGenerator** - Polish analogique
10. **SurgeReverb** - Optionnel

---

## Modules Déjà Implémentés

### Oscillateurs ✅ (11/11)
- [x] AnalogOscillator (`analog.rs`)
- [x] WavetableOsc1D (`wavetable.rs`)
- [x] WavetableOsc2D (`wavetable2d.rs`)
- [x] FMOscillator (`fm.rs`)
- [x] PMOscillator (`pm.rs`)
- [x] NoiseOscillator (`noise.rs`)
- [x] LFO (`lfo.rs`)
- [x] VectorOscillator (`vector.rs`)
- [x] ChiptuneOscillator (`chiptune.rs`)
- [x] MultiOscillator (`multi.rs`)
- [x] DriftGenerator (`drift.rs`)

### Filtres ✅ (10/10)
- [x] LadderFilter (`ladder.rs`)
- [x] DiodeFilter (`diode.rs`)
- [x] SEMFilter (`sem.rs`)
- [x] Korg35Filter (`korg35.rs`)
- [x] CombFilter (`comb.rs`)
- [x] FormantFilter (`formant.rs`)
- [x] BiquadFilter/Allpass/Resonator (`biquad.rs`)
- [x] VAOnePoleFilter (`va_one_pole.rs`)
- [x] DCBlockingFilter (`dc_blocker.rs`)
- [x] BiquadEQ (`eq.rs`)

### Effets (12/13)
- [x] Delay (`delay.rs`)
- [x] Chorus (`chorus.rs`)
- [x] Phaser (`phaser.rs`)
- [x] Flanger (`flanger.rs`)
- [x] RingModulator (`ring_mod.rs`)
- [x] OversamplingDistortion (`distortion.rs`)
- [x] Bitcrusher (`bitcrusher.rs`)
- [x] ParametricEQ (`parametric_eq.rs`)
- [x] ZitaReverb (`reverb.rs`)
- [ ] SurgeReverb (optionnel)

### Autres ✅
- [x] ADSR Envelope (`adsr.rs`)
- [x] Voice (`voice.rs`)
- [x] Engine (`engine.rs`)
- [x] ModMatrix (`mod_matrix.rs`)
- [x] Wavetables (160 tables converties)
