use {
    crate::ToSignal,
    bevy::reflect::{TypePath, TypeUuid},
};

/// [`Asset`](bevy::asset::Asset) form of [`Sine`](oddio::Sine)
#[derive(TypeUuid, TypePath)]
#[uuid = "14597aba-d411-4bfc-b227-09cf5f88202f"]
pub struct Sine;

/// Settings for initialization of [`Sine`] audio source.
/// See [`Sine::new`](oddio::Sine::new) for more information.
pub struct Settings {
    /// The initial phase of the sine wave in radians.
    pub phase: f32,
    /// The frequence of the sine wave in Hz.
    pub frequency_hz: f32,
}

impl Settings {
    /// Generate settings for [`Sine`].
    #[must_use]
    pub fn new(phase: f32, frequency_hz: f32) -> Self {
        Self {
            phase,
            frequency_hz,
        }
    }
}

impl ToSignal for Sine {
    type Settings = Settings;
    type Signal = oddio::Sine;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::Sine::new(settings.phase, settings.frequency_hz)
    }
}
