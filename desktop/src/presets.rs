use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use synth::{
    Chorus, EnvelopeSettings, FilterMode, FilterSettings, Flanger, Hammer, Instrument,
    OscillatorAssignment, Reverb, Tremolo, Vibrato, Waveform,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename = "presets")]
pub struct PresetBank {
    #[serde(rename = "preset", default)]
    pub presets: Vec<Preset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "oscillator", default)]
    pub oscillators: Vec<OscillatorPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hammer: Option<HammerPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vibrato: Option<VibratoPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<TremoloPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chorus: Option<ChorusPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flanger: Option<FlangerPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverb: Option<ReverbPreset>,
    #[serde(default)]
    pub envelope: EnvelopePreset,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OscillatorPreset {
    #[serde(rename = "@waveform")]
    pub waveform: WaveformPreset,
    #[serde(rename = "@gain")]
    pub gain: f32,
    #[serde(rename = "@frequency_ratio", default = "default_frequency_ratio")]
    pub frequency_ratio: f32,
    #[serde(rename = "@detune_cents", default)]
    pub detune_cents: f32,
    #[serde(rename = "@decay_seconds", default = "default_partial_decay")]
    pub decay_seconds: f32,
    #[serde(rename = "@velocity_sensitivity", default)]
    pub velocity_sensitivity: f32,
}

fn default_frequency_ratio() -> f32 {
    1.0
}
fn default_partial_decay() -> f32 {
    60.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HammerPreset {
    #[serde(rename = "@gain")]
    pub gain: f32,
    #[serde(rename = "@decay_seconds")]
    pub decay_seconds: f32,
    #[serde(rename = "@cutoff_hz")]
    pub cutoff_hz: f32,
    #[serde(rename = "@velocity_sensitivity")]
    pub velocity_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WaveformPreset {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

impl WaveformPreset {
    pub const ALL: [Self; 4] = [Self::Sine, Self::Square, Self::Triangle, Self::Sawtooth];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Square => "Square",
            Self::Triangle => "Triangle",
            Self::Sawtooth => "Sawtooth",
        }
    }

    const fn into_domain(self) -> Waveform {
        match self {
            Self::Sine => Waveform::Sine,
            Self::Square => Waveform::Square,
            Self::Triangle => Waveform::Triangle,
            Self::Sawtooth => Waveform::Sawtooth,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterPreset {
    #[serde(rename = "@mode")]
    pub mode: FilterModePreset,
    #[serde(rename = "@cutoff_hz")]
    pub cutoff_hz: f32,
    #[serde(rename = "@resonance_q")]
    pub resonance_q: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FilterModePreset {
    LowPass,
    BandPass,
    HighPass,
}

impl FilterModePreset {
    pub const ALL: [Self; 3] = [Self::LowPass, Self::BandPass, Self::HighPass];

    pub const fn label(self) -> &'static str {
        match self {
            Self::LowPass => "Low-pass",
            Self::BandPass => "Band-pass",
            Self::HighPass => "High-pass",
        }
    }

    const fn into_domain(self) -> FilterMode {
        match self {
            Self::LowPass => FilterMode::LowPass,
            Self::BandPass => FilterMode::BandPass,
            Self::HighPass => FilterMode::HighPass,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VibratoPreset {
    #[serde(rename = "@rate_hz")]
    pub rate_hz: f32,
    #[serde(rename = "@depth_cents")]
    pub depth_cents: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TremoloPreset {
    #[serde(rename = "@rate_hz")]
    pub rate_hz: f32,
    #[serde(rename = "@depth")]
    pub depth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChorusPreset {
    #[serde(rename = "@rate_hz")]
    pub rate_hz: f32,
    #[serde(rename = "@base_delay_ms")]
    pub base_delay_ms: f32,
    #[serde(rename = "@depth_ms")]
    pub depth_ms: f32,
    #[serde(rename = "@mix")]
    pub mix: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlangerPreset {
    #[serde(rename = "@rate_hz")]
    pub rate_hz: f32,
    #[serde(rename = "@base_delay_ms")]
    pub base_delay_ms: f32,
    #[serde(rename = "@depth_ms")]
    pub depth_ms: f32,
    #[serde(rename = "@feedback")]
    pub feedback: f32,
    #[serde(rename = "@mix")]
    pub mix: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReverbPreset {
    #[serde(rename = "@room_size")]
    pub room_size: f32,
    #[serde(rename = "@damping")]
    pub damping: f32,
    #[serde(rename = "@mix")]
    pub mix: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvelopePreset {
    #[serde(rename = "@attack_seconds")]
    pub attack_seconds: f32,
    #[serde(rename = "@decay_seconds")]
    pub decay_seconds: f32,
    #[serde(rename = "@sustain_level")]
    pub sustain_level: f32,
    #[serde(rename = "@release_seconds")]
    pub release_seconds: f32,
}

impl Default for EnvelopePreset {
    fn default() -> Self {
        Self {
            attack_seconds: 0.1,
            decay_seconds: 0.2,
            sustain_level: 0.7,
            release_seconds: 0.5,
        }
    }
}

impl PresetBank {
    pub fn load(path: &Path) -> Result<Self, String> {
        let xml = fs::read_to_string(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        Self::from_xml(&xml)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let xml = self.to_xml()?;
        fs::write(
            path,
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{xml}\n"),
        )
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
    }

    pub fn from_xml(xml: &str) -> Result<Self, String> {
        let bank: Self =
            quick_xml::de::from_str(xml).map_err(|error| format!("Invalid preset XML: {error}"))?;
        bank.to_instruments()?;
        Ok(bank)
    }

    pub fn to_xml(&self) -> Result<String, String> {
        quick_xml::se::to_string(self)
            .map_err(|error| format!("Could not serialize presets: {error}"))
    }

    pub fn to_instruments(&self) -> Result<Vec<Instrument>, String> {
        if self.presets.is_empty() {
            return Err("The preset bank must contain at least one preset".into());
        }

        self.presets
            .iter()
            .enumerate()
            .map(|(index, preset)| preset.to_instrument(index))
            .collect()
    }
}

impl Preset {
    fn to_instrument(&self, index: usize) -> Result<Instrument, String> {
        let context = || format!("Preset {} ({})", index + 1, self.name);
        if self.name.trim().is_empty() {
            return Err(format!("Preset {} has an empty name", index + 1));
        }
        if self.oscillators.is_empty() {
            return Err(format!("{} needs at least one oscillator", context()));
        }
        if self.oscillators.iter().any(|oscillator| {
            !oscillator.gain.is_finite()
                || oscillator.gain < 0.0
                || !oscillator.frequency_ratio.is_finite()
                || oscillator.frequency_ratio <= 0.0
                || !oscillator.detune_cents.is_finite()
                || !oscillator.decay_seconds.is_finite()
                || oscillator.decay_seconds <= 0.0
                || !oscillator.velocity_sensitivity.is_finite()
                || oscillator.velocity_sensitivity < 0.0
        }) {
            return Err(format!("{} has an invalid oscillator gain", context()));
        }

        validate_non_negative(self.envelope.attack_seconds, "attack", &context())?;
        validate_non_negative(self.envelope.decay_seconds, "decay", &context())?;
        validate_range(self.envelope.sustain_level, 0.0, 1.0, "sustain", &context())?;
        validate_non_negative(self.envelope.release_seconds, "release", &context())?;

        let oscillators = self
            .oscillators
            .iter()
            .map(|oscillator| {
                OscillatorAssignment::new(oscillator.waveform.into_domain(), oscillator.gain)
                    .with_partial(
                        oscillator.frequency_ratio,
                        oscillator.detune_cents,
                        oscillator.decay_seconds,
                        oscillator.velocity_sensitivity,
                    )
            })
            .collect();
        let mut instrument =
            Instrument::new(self.name.clone(), oscillators).with_envelope(EnvelopeSettings::new(
                self.envelope.attack_seconds,
                self.envelope.decay_seconds,
                self.envelope.sustain_level,
                self.envelope.release_seconds,
            ));

        if let Some(hammer) = &self.hammer {
            validate_non_negative(hammer.gain, "hammer gain", &context())?;
            validate_positive(hammer.decay_seconds, "hammer decay", &context())?;
            validate_positive(hammer.cutoff_hz, "hammer cutoff", &context())?;
            validate_non_negative(
                hammer.velocity_sensitivity,
                "hammer velocity sensitivity",
                &context(),
            )?;
            instrument = instrument.with_hammer(Hammer::new(
                hammer.gain,
                hammer.decay_seconds,
                hammer.cutoff_hz,
                hammer.velocity_sensitivity,
            ));
        }

        if let Some(filter) = &self.filter {
            validate_positive(filter.cutoff_hz, "filter cutoff", &context())?;
            validate_range(filter.resonance_q, 0.5, 20.0, "filter Q", &context())?;
            instrument = instrument.with_filter(FilterSettings::new(
                filter.mode.into_domain(),
                filter.cutoff_hz,
                filter.resonance_q,
            ));
        }
        if let Some(vibrato) = &self.vibrato {
            validate_non_negative(vibrato.rate_hz, "vibrato rate", &context())?;
            validate_non_negative(vibrato.depth_cents, "vibrato depth", &context())?;
            instrument =
                instrument.with_vibrato(Vibrato::new(vibrato.rate_hz, vibrato.depth_cents));
        }
        if let Some(tremolo) = &self.tremolo {
            validate_non_negative(tremolo.rate_hz, "tremolo rate", &context())?;
            validate_range(tremolo.depth, 0.0, 1.0, "tremolo depth", &context())?;
            instrument = instrument.with_tremolo(Tremolo::new(tremolo.rate_hz, tremolo.depth));
        }
        if let Some(chorus) = &self.chorus {
            validate_non_negative(chorus.rate_hz, "chorus rate", &context())?;
            validate_positive(chorus.base_delay_ms, "chorus base delay", &context())?;
            validate_non_negative(chorus.depth_ms, "chorus depth", &context())?;
            validate_range(chorus.mix, 0.0, 1.0, "chorus mix", &context())?;
            instrument = instrument.with_chorus(Chorus::new(
                chorus.rate_hz,
                chorus.base_delay_ms,
                chorus.depth_ms,
                chorus.mix,
            ));
        }
        if let Some(flanger) = &self.flanger {
            validate_non_negative(flanger.rate_hz, "flanger rate", &context())?;
            validate_positive(flanger.base_delay_ms, "flanger base delay", &context())?;
            validate_non_negative(flanger.depth_ms, "flanger depth", &context())?;
            validate_range(
                flanger.feedback,
                -0.95,
                0.95,
                "flanger feedback",
                &context(),
            )?;
            validate_range(flanger.mix, 0.0, 1.0, "flanger mix", &context())?;
            instrument = instrument.with_flanger(Flanger::new(
                flanger.rate_hz,
                flanger.base_delay_ms,
                flanger.depth_ms,
                flanger.feedback,
                flanger.mix,
            ));
        }
        if let Some(reverb) = &self.reverb {
            validate_range(reverb.room_size, 0.0, 1.0, "reverb room size", &context())?;
            validate_range(reverb.damping, 0.0, 1.0, "reverb damping", &context())?;
            validate_range(reverb.mix, 0.0, 1.0, "reverb mix", &context())?;
            instrument =
                instrument.with_reverb(Reverb::new(reverb.room_size, reverb.damping, reverb.mix));
        }

        Ok(instrument)
    }
}

fn validate_non_negative(value: f32, field: &str, context: &str) -> Result<(), String> {
    validate_range(value, 0.0, f32::MAX, field, context)
}

fn validate_positive(value: f32, field: &str, context: &str) -> Result<(), String> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(format!("{context} has an invalid {field}"))
    }
}

fn validate_range(
    value: f32,
    minimum: f32,
    maximum: f32,
    field: &str,
    context: &str,
) -> Result<(), String> {
    if value.is_finite() && (minimum..=maximum).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "{context} has an invalid {field}; expected {minimum}..={maximum}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"
        <presets>
          <preset name="Warm">
            <oscillator waveform="triangle" gain="0.7" frequency_ratio="2.01" detune_cents="1.5" decay_seconds="1.2" velocity_sensitivity="0.8"/>
            <oscillator waveform="sine" gain="0.3"/>
            <hammer gain="0.06" decay_seconds="0.018" cutoff_hz="6000" velocity_sensitivity="1.5"/>
            <filter mode="lowpass" cutoff_hz="2500" resonance_q="0.707"/>
            <vibrato rate_hz="5" depth_cents="12"/>
            <tremolo rate_hz="4" depth="0.2"/>
            <chorus rate_hz="0.6" base_delay_ms="20" depth_ms="5" mix="0.3"/>
            <flanger rate_hz="0.2" base_delay_ms="1" depth_ms="2" feedback="0.5" mix="0.25"/>
            <reverb room_size="0.65" damping="0.4" mix="0.2"/>
            <envelope attack_seconds="0.1" decay_seconds="0.2" sustain_level="0.7" release_seconds="0.5"/>
          </preset>
        </presets>
    "#;

    #[test]
    fn loads_and_converts_xml_presets() {
        let bank = PresetBank::from_xml(XML).unwrap();
        let instruments = bank.to_instruments().unwrap();

        assert_eq!(instruments[0].name(), "Warm");
        assert_eq!(instruments[0].oscillators().len(), 2);
        assert_eq!(instruments[0].oscillators()[0].frequency_ratio(), 2.01);
        assert!(instruments[0].hammer().is_some());
        assert!(instruments[0].filter().is_some());
        assert!(instruments[0].vibrato().is_some());
        assert!(instruments[0].tremolo().is_some());
        assert!(instruments[0].chorus().is_some());
        assert!(instruments[0].flanger().is_some());
        assert!(instruments[0].reverb().is_some());
    }

    #[test]
    fn round_trips_xml_presets() {
        let bank = PresetBank::from_xml(XML).unwrap();
        let serialized = bank.to_xml().unwrap();
        let restored = PresetBank::from_xml(&serialized).unwrap();

        assert_eq!(restored, bank);
    }

    #[test]
    fn loads_the_bundled_preset_bank() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("presets.xml");
        let bank = PresetBank::load(&path).unwrap();

        assert_eq!(bank.presets.len(), 6);
        assert_eq!(bank.presets[4].name, "Warm");
        assert_eq!(bank.presets[5].name, "Classical Piano");
    }
}
