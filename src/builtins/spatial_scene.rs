use {
    crate::ToSignal,
    bevy::reflect::{TypePath, TypeUuid},
};

/// [`Asset`](bevy::asset::Asset) form of [`SpatialScene`](oddio::SpatialScene)
#[derive(TypeUuid, TypePath)]
#[uuid = "4c5ea5bb-293e-485e-93d9-7a4f69d2130a"]
pub struct SpatialScene;

impl ToSignal for SpatialScene {
    type Settings = ();
    type Signal = oddio::SpatialScene;

    fn to_signal(&self, _settings: Self::Settings) -> Self::Signal {
        oddio::SpatialScene::new()
    }
}
