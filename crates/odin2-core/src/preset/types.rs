//! Preset type definitions
//!
//! These types define the structure of an Odin 2 preset.

/// Oscillator type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum OscillatorType {
    #[default]
    Analog = 0,
    Wavetable = 1,
    Multi = 2,
    Vector = 3,
    Chiptune = 4,
    FM = 5,
    PM = 6,
    Noise = 7,
    Wavedraw = 8,
    Chipdraw = 9,
    Specdraw = 10,
}

impl From<i32> for OscillatorType {
    fn from(v: i32) -> Self {
        match v {
            0 => OscillatorType::Analog,
            1 => OscillatorType::Wavetable,
            2 => OscillatorType::Multi,
            3 => OscillatorType::Vector,
            4 => OscillatorType::Chiptune,
            5 => OscillatorType::FM,
            6 => OscillatorType::PM,
            7 => OscillatorType::Noise,
            8 => OscillatorType::Wavedraw,
            9 => OscillatorType::Chipdraw,
            10 => OscillatorType::Specdraw,
            _ => OscillatorType::Analog,
        }
    }
}

/// Analog waveform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AnalogWaveform {
    #[default]
    Saw = 0,
    Pulse = 1,
    Triangle = 2,
    Sine = 3,
}

impl From<i32> for AnalogWaveform {
    fn from(v: i32) -> Self {
        match v {
            0 => AnalogWaveform::Saw,
            1 => AnalogWaveform::Pulse,
            2 => AnalogWaveform::Triangle,
            3 => AnalogWaveform::Sine,
            _ => AnalogWaveform::Saw,
        }
    }
}

/// Filter type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FilterType {
    #[default]
    LP24 = 0,
    LP12 = 1,
    BP24 = 2,
    BP12 = 3,
    HP24 = 4,
    HP12 = 5,
    SEM12 = 6,
    Diode = 7,
    Korg = 8,
    Comb = 9,
    Formant = 10,
    RingMod = 11,
}

impl From<i32> for FilterType {
    fn from(v: i32) -> Self {
        match v {
            0 => FilterType::LP24,
            1 => FilterType::LP12,
            2 => FilterType::BP24,
            3 => FilterType::BP12,
            4 => FilterType::HP24,
            5 => FilterType::HP12,
            6 => FilterType::SEM12,
            7 => FilterType::Diode,
            8 => FilterType::Korg,
            9 => FilterType::Comb,
            10 => FilterType::Formant,
            11 => FilterType::RingMod,
            _ => FilterType::LP24,
        }
    }
}

/// LFO waveform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LfoWaveform {
    #[default]
    Sine = 0,
    Saw = 1,
    SawDown = 2,
    Square = 3,
    Triangle = 4,
    SampleAndHold = 5,
}

impl From<i32> for LfoWaveform {
    fn from(v: i32) -> Self {
        match v {
            0 => LfoWaveform::Sine,
            1 => LfoWaveform::Saw,
            2 => LfoWaveform::SawDown,
            3 => LfoWaveform::Square,
            4 => LfoWaveform::Triangle,
            5 => LfoWaveform::SampleAndHold,
            _ => LfoWaveform::Sine,
        }
    }
}

/// Distortion algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DistortionAlgo {
    #[default]
    Tanh = 0,
    HardClip = 1,
    Saturate = 2,
    Foldback = 3,
    Sine = 4,
}

impl From<i32> for DistortionAlgo {
    fn from(v: i32) -> Self {
        match v {
            0 => DistortionAlgo::Tanh,
            1 => DistortionAlgo::HardClip,
            2 => DistortionAlgo::Saturate,
            3 => DistortionAlgo::Foldback,
            4 => DistortionAlgo::Sine,
            _ => DistortionAlgo::Tanh,
        }
    }
}

/// Play mode (mono/poly/legato)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PlayMode {
    Legato = 0,
    #[default]
    Poly = 1,
    Mono = 2,
}

impl From<i32> for PlayMode {
    fn from(v: i32) -> Self {
        match v {
            0 => PlayMode::Legato,
            1 => PlayMode::Poly,
            2 => PlayMode::Mono,
            _ => PlayMode::Poly,
        }
    }
}

/// Arpeggiator direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ArpDirection {
    #[default]
    Up = 0,
    Down = 1,
    UpDown = 2,
    DownUp = 3,
    Random = 4,
}

impl From<i32> for ArpDirection {
    fn from(v: i32) -> Self {
        match v {
            0 => ArpDirection::Up,
            1 => ArpDirection::Down,
            2 => ArpDirection::UpDown,
            3 => ArpDirection::DownUp,
            4 => ArpDirection::Random,
            _ => ArpDirection::Up,
        }
    }
}

/// Modulation source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ModSource {
    #[default]
    None = 0,
    Lfo1 = 1,
    Lfo2 = 2,
    Lfo3 = 3,
    Lfo4 = 4,
    Env1 = 5,
    Env2 = 6,
    Env3 = 7,
    Env4 = 8,
    Velocity = 100,
    Note = 101,
    Aftertouch = 102,
    ModWheel = 103,
    X = 104,
    Y = 105,
    Random = 106,
    // ... more sources
}

impl From<i32> for ModSource {
    fn from(v: i32) -> Self {
        match v {
            0 => ModSource::None,
            1 => ModSource::Lfo1,
            2 => ModSource::Lfo2,
            3 => ModSource::Lfo3,
            4 => ModSource::Lfo4,
            5 => ModSource::Env1,
            6 => ModSource::Env2,
            7 => ModSource::Env3,
            8 => ModSource::Env4,
            100 => ModSource::Velocity,
            101 => ModSource::Note,
            102 => ModSource::Aftertouch,
            103 => ModSource::ModWheel,
            104 => ModSource::X,
            105 => ModSource::Y,
            106 => ModSource::Random,
            _ => ModSource::None,
        }
    }
}

/// Oscillator parameters
#[derive(Debug, Clone, Default)]
pub struct OscillatorParams {
    pub osc_type: OscillatorType,
    pub analog_wave: AnalogWaveform,
    pub wavetable: i32,
    pub octave: i32,
    pub semitones: i32,
    pub fine: f32,
    pub volume: f32,
    pub reset: bool,
    pub sync: bool,
    pub position: f32,
    pub detune: f32,
    pub pos_mod: f32,
    pub multi_position: f32,
    pub spread: f32,
    pub pulsewidth: f32,
    pub drift: f32,
    pub vec_x: f32,
    pub vec_y: f32,
    pub vec_a: i32,
    pub vec_b: i32,
    pub vec_c: i32,
    pub vec_d: i32,
    pub chipwave: i32,
    pub arp_on: bool,
    pub arp_speed: f32,
    pub step_1: i32,
    pub step_2: i32,
    pub step_3: i32,
    pub step_3_on: bool,
    pub chipnoise: bool,
    pub exp_fm: bool,
    pub fm: f32,
    pub lp: f32,
    pub hp: f32,
    pub modulator_wave: i32,
    pub carrier_wave: i32,
    pub mod_source: i32,
    pub carrier_ratio: i32,
    pub modulator_ratio: i32,
}

/// Filter parameters
#[derive(Debug, Clone, Default)]
pub struct FilterParams {
    pub filter_type: FilterType,
    pub frequency: f32,
    pub resonance: f32,
    pub saturation: f32,
    pub gain: f32,
    pub env_amount: f32,
    pub vel_amount: f32,
    pub kbd_follow: f32,
    pub sem_transition: f32,
    pub formant_transition: f32,
    pub ring_mod_amount: f32,
    pub comb_polarity: bool,
    pub vowel_left: i32,
    pub vowel_right: i32,
    pub osc1_input: bool,
    pub osc2_input: bool,
    pub osc3_input: bool,
    pub fil1_input: bool,
    pub to_amp: bool,
}

/// ADSR envelope parameters
#[derive(Debug, Clone, Default)]
pub struct EnvelopeParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub loop_env: bool,
}

/// LFO parameters
#[derive(Debug, Clone, Default)]
pub struct LfoParams {
    pub waveform: LfoWaveform,
    pub frequency: f32,
    pub reset: bool,
    pub sync: bool,
    pub sync_numerator: i32,
    pub sync_denominator: i32,
}

/// Delay effect parameters
#[derive(Debug, Clone, Default)]
pub struct DelayParams {
    pub on: bool,
    pub time: f32,
    pub feedback: f32,
    pub dry: f32,
    pub wet: f32,
    pub hp: f32,
    pub ducking: f32,
    pub pingpong: bool,
    pub sync: bool,
}

/// Reverb effect parameters
#[derive(Debug, Clone, Default)]
pub struct ReverbParams {
    pub on: bool,
    pub delay: f32,
    pub dry_wet: f32,
    pub mid_hall: f32,
    pub hf_damp: f32,
    pub eq_gain: f32,
    pub eq_freq: f32,
}

/// Chorus effect parameters
#[derive(Debug, Clone, Default)]
pub struct ChorusParams {
    pub on: bool,
    pub amount: f32,
    pub rate: f32,
    pub feedback: f32,
    pub dry_wet: f32,
    pub reset: bool,
    pub sync: bool,
}

/// Phaser effect parameters
#[derive(Debug, Clone, Default)]
pub struct PhaserParams {
    pub on: bool,
    pub frequency: f32,
    pub feedback: f32,
    pub rate: f32,
    pub mod_amount: f32,
    pub dry_wet: f32,
    pub reset: bool,
    pub sync: bool,
}

/// Flanger effect parameters
#[derive(Debug, Clone, Default)]
pub struct FlangerParams {
    pub on: bool,
    pub amount: f32,
    pub rate: f32,
    pub feedback: f32,
    pub dry_wet: f32,
    pub reset: bool,
    pub sync: bool,
}

/// Distortion effect parameters
#[derive(Debug, Clone, Default)]
pub struct DistortionParams {
    pub on: bool,
    pub algorithm: DistortionAlgo,
    pub boost: f32,
    pub dry_wet: f32,
}

/// Modulation matrix row
#[derive(Debug, Clone, Default)]
pub struct ModMatrixRow {
    pub source: i32,
    pub dest1: i32,
    pub dest2: i32,
    pub scale: i32,
    pub amount0: f32,
    pub amount1: f32,
    pub amount2: f32,
}

/// Arpeggiator step
#[derive(Debug, Clone, Default)]
pub struct ArpStep {
    pub on: bool,
    pub transpose: i32,
    pub mod1: f32,
    pub mod2: f32,
}

/// Arpeggiator parameters
#[derive(Debug, Clone, Default)]
pub struct ArpeggiatorParams {
    pub on: bool,
    pub one_shot: bool,
    pub direction: ArpDirection,
    pub octaves: i32,
    pub steps: i32,
    pub gate: i32,
    pub mod_transpose: f32,
    pub sync_numerator: i32,
    pub sync_denominator: i32,
    pub step_params: [ArpStep; 16],
}
