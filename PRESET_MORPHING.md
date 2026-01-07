# Preset Morphing & Emotional Sound Design

## Vue d'ensemble

Le système de morphing de presets d'odin-rs permet de créer des sons dynamiques qui évoluent en temps réel en fonction de l'état émotionnel du jeu, du joueur, ou de toute autre variable de votre application.

## Presets Procéduraux Prêts à l'Emploi

### 🎭 Les 4 Émotions de Base

Aucun fichier `.odin` externe n'est nécessaire ! Les presets émotionnels sont créés procéduralement :

```rust
use odin2_core::preset::OdinPreset;

// Créer les 4 presets émotionnels
let happy = OdinPreset::create_happy();    // Lumineux, énergique, attaque rapide
let sad = OdinPreset::create_sad();        // Sombre, lent, mélancolique
let angry = OdinPreset::create_angry();    // Agressif, dissonant, percutant
let calm = OdinPreset::create_calm();      // Doux, équilibré, apaisant
```

#### Caractéristiques de chaque preset :

| Preset | Filtre | Enveloppe | Oscillateurs | Usage |
|--------|--------|-----------|--------------|-------|
| **Happy** | 8000 Hz (bright) | Attack: 0.01s (rapide) | Octaves hautes, saw/pulse | Victoire, joie, succès |
| **Sad** | 800 Hz (dark) | Attack: 0.5s (lent) | Octave basse, triangle/sine | Défaite, tristesse, perte |
| **Angry** | 3000 Hz + resonance | Attack: 0.001s (percutant) | Tritone (dissonant), saw | Combat, tension, danger |
| **Calm** | 2500 Hz (mid) | Attack: 0.2s (doux) | Accord majeur, triangle/sine | Exploration, repos, sécurité |

## Morphing 1D : Entre Deux Émotions

### Interpolation Linéaire

```rust
let happy = OdinPreset::create_happy();
let sad = OdinPreset::create_sad();

// Morpher entre happy et sad (t de 0.0 à 1.0)
let emotion = player_emotion_score(); // 0.0 = happy, 1.0 = sad
let morphed = happy.interpolate(&sad, emotion);

// Charger dans l'engine
let mut engine = OdinEngine::new(44100.0);
engine.load_preset(&morphed);

// Jouer une note
engine.note_on(60, 100); // C4
```

### Interpolation Lisse (avec Easing)

Pour des transitions plus naturelles, utilisez `interpolate_smooth()` :

```rust
let morphed = happy.interpolate_smooth(&sad, emotion);
```

Cela applique une courbe d'easing (smoothstep) qui adoucit les transitions.

## Morphing 2D : Espace Émotionnel Complet

### Modèle Circumplex (Valence × Arousal)

Le système supporte un espace émotionnel 2D basé sur le modèle psychologique de Russell :

```
        Arousal (Énergie)
              ↑
              │
      Angry   │   Happy
    (tension) │ (joie)
              │
─────────────┼────────────→ Valence
  (négatif)  │  (positif)
              │
       Sad    │   Calm
   (tristesse)│ (paix)
              │
```

### Utilisation :

```rust
// Valence: 0.0 (négatif) → 1.0 (positif)
// Arousal: 0.0 (basse énergie) → 1.0 (haute énergie)

let valence = player_happiness();  // 0.0 à 1.0
let arousal = player_energy();     // 0.0 à 1.0

let sound = OdinPreset::create_emotional_2d(valence, arousal);
engine.load_preset(&sound);
```

### Exemples de Positions 2D :

| Position | Valence | Arousal | Émotion Résultante |
|----------|---------|---------|-------------------|
| (0.0, 0.0) | Négatif | Bas | Triste, déprimé |
| (1.0, 0.0) | Positif | Bas | Calme, paisible |
| (0.0, 1.0) | Négatif | Haut | Anxieux, énervé |
| (1.0, 1.0) | Positif | Haut | Joyeux, excité |
| (0.5, 0.5) | Neutre | Moyen | Neutre, équilibré |
| (0.8, 0.3) | Positif | Bas | Satisfait, serein |
| (0.2, 0.7) | Négatif | Haut | Frustré, stressé |

## Exemples d'Intégration de Jeu

### Pattern 1 : Musique Adaptive Simple

```rust
struct AdaptiveMusic {
    engine: OdinEngine,
    happy: OdinPreset,
    sad: OdinPreset,
    current_emotion: f32,
}

impl AdaptiveMusic {
    fn new() -> Self {
        Self {
            engine: OdinEngine::new(44100.0),
            happy: OdinPreset::create_happy(),
            sad: OdinPreset::create_sad(),
            current_emotion: 0.5,
        }
    }

    fn update(&mut self, player_health: f32) {
        // health: 1.0 = happy, 0.0 = sad
        let emotion = 1.0 - player_health;

        // Éviter les changements trop fréquents
        if (emotion - self.current_emotion).abs() > 0.05 {
            let morphed = self.happy.interpolate(&self.sad, emotion);
            self.engine.load_preset(&morphed);
            self.current_emotion = emotion;
        }
    }

    fn play_note(&mut self, note: u8, velocity: u8) {
        self.engine.note_on(note, velocity);
    }
}
```

### Pattern 2 : Combat Dynamique (2D)

```rust
struct CombatMusic {
    engine: OdinEngine,
}

impl CombatMusic {
    fn update_from_combat(&mut self, is_winning: bool, intensity: f32) {
        let valence = if is_winning { 0.8 } else { 0.2 };
        let arousal = intensity; // 0.0 (calme) à 1.0 (intense)

        let sound = OdinPreset::create_emotional_2d(valence, arousal);
        self.engine.load_preset(&sound);
    }
}

// Usage dans le jeu :
fn on_combat_update(music: &mut CombatMusic, player: &Player, enemy: &Enemy) {
    let is_winning = player.health > enemy.health;
    let intensity = (player.damage_dealt + enemy.damage_dealt) / 100.0;
    music.update_from_combat(is_winning, intensity.clamp(0.0, 1.0));
}
```

### Pattern 3 : Exploration avec Danger

```rust
fn update_exploration_music(
    engine: &mut OdinEngine,
    player_position: Vec2,
    enemy_positions: &[Vec2]
) {
    // Calculer la distance à l'ennemi le plus proche
    let min_distance = enemy_positions.iter()
        .map(|&pos| (pos - player_position).length())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1000.0);

    // Distance sûre : calm (1.0, 0.0)
    // Danger proche : anxious (0.2, 0.9)
    let valence = (min_distance / 100.0).clamp(0.2, 1.0);
    let arousal = (1.0 - min_distance / 100.0).clamp(0.0, 0.9);

    let sound = OdinPreset::create_emotional_2d(valence, arousal);
    engine.load_preset(&sound);
}
```

## API Complète

### Création de Presets

```rust
// Presets procéduraux
OdinPreset::create_happy() -> OdinPreset
OdinPreset::create_sad() -> OdinPreset
OdinPreset::create_angry() -> OdinPreset
OdinPreset::create_calm() -> OdinPreset

// Espace émotionnel 2D
OdinPreset::create_emotional_2d(valence: f32, arousal: f32) -> OdinPreset

// Charger depuis un fichier .odin
OdinPreset::load(path: impl AsRef<Path>) -> Result<OdinPreset, ValueTreeError>
```

### Morphing

```rust
// Interpolation linéaire (t entre 0.0 et 1.0)
preset_a.interpolate(&preset_b, t: f32) -> OdinPreset

// Interpolation lisse (avec easing)
preset_a.interpolate_smooth(&preset_b, t: f32) -> OdinPreset
```

### Intégration Engine

```rust
// Charger un preset dans l'engine
engine.load_preset(&preset: &OdinPreset)

// Le preset est appliqué à toutes les nouvelles notes
engine.note_on(note: u8, velocity: u8)
```

## Tester Vous-Même

### Exemple Complet

```bash
cargo run --example preset_morph --features std
```

Cet exemple démontre :
- Création des 4 presets émotionnels
- Morphing à différents niveaux (0%, 30%, 50%, 100%)
- Interpolation lisse vs linéaire
- Espace émotionnel 2D
- Génération audio avec le morphing

### Tests Unitaires

```bash
cargo test --features std test_procedural
cargo test --features std test_emotional_2d
```

## Performance

- **Création de preset** : ~1 µs (très rapide)
- **Interpolation** : ~5 µs pour un preset complet (200+ paramètres)
- **Chargement dans l'engine** : ~10 µs
- **Temps réel** : ✅ Peut être fait à chaque frame sans problème

## Limitations Actuelles

Le pont minimal vers l'engine ne supporte actuellement que :
- ✅ Oscillateurs analogiques (saw, pulse, triangle, sine)
- ✅ Filtre principal avec modulation d'enveloppe
- ✅ Enveloppes ADSR complètes
- ❌ Oscillateurs wavetable/FM/vector (futurs)
- ❌ Effets globaux (delay, reverb) (futurs)
- ❌ LFOs (futurs)

Ces limitations seront levées dans les versions futures, mais les oscillateurs analogiques suffisent pour une large gamme de sons émotionnels.

## Ressources Additionnelles

- [Russell's Circumplex Model of Affect](https://en.wikipedia.org/wiki/Emotion_classification#Circumplex_model)
- [Odin 2 Synthesizer](https://www.thewavewarden.com/odin2)
- [Procedural Audio in Games](https://designingsound.org/tag/procedural-audio/)

## Licence

Ce code fait partie du projet odin-rs et est sous licence MIT.
