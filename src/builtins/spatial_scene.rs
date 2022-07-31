use bevy::reflect::TypeUuid;

use crate::ToSignal;

/// [`Asset`](bevy::asset::Asset) form of [`SpatialScene`](oddio::SpatialScene)
#[derive(TypeUuid)]
#[uuid = "4c5ea5bb-293e-485e-93d9-7a4f69d2130a"]
pub struct SpatialScene;

/// Settings for initialization of [`SpatialScene`] audio source.
/// See [`SpatialScene::new`](oddio::SpatialScene::new) for more information.
pub struct Settings {
    /// The sample rate.
    pub rate: u32,
    /// The duration of the buffer.
    pub buffer_duration: f32,
}

impl Settings {
    /// Generate settings for [`SpatialScene`].
    #[must_use]
    pub fn new(rate: u32, buffer_duration: f32) -> Self {
        Self {
            rate,
            buffer_duration,
        }
    }
}

impl ToSignal for SpatialScene {
    type Settings = Settings;
    type Signal = oddio::SpatialScene;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::SpatialScene::new(settings.rate, settings.buffer_duration)
    }
}
