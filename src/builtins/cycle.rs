use {
    crate::ToSignal,
    bevy::reflect::TypeUuid,
    oddio::{Frame, Frames},
    std::{marker::PhantomData, sync::Arc},
};

/// [`Asset`](bevy::asset::Asset) form of [`Constant`](oddio::Constant)
#[derive(TypeUuid, Default)]
#[uuid = "f391d20f-7654-403a-b7c9-3f3c7991138a"]
pub struct Cycle<T> {
    _phantom: PhantomData<T>,
}

impl<T> Cycle<T> {
    /// Generate new `Constant` source
    #[must_use]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Settings for initialization of [`Cycle`] audio source.
/// See [`Cycle::new`](oddio::Cycle::new) for more information.
pub struct Settings<T> {
    frames: Arc<Frames<T>>,
}

impl<T> Settings<T> {
    /// Generate settings for [`Cycle`].
    #[must_use]
    pub fn new(frames: Arc<Frames<T>>) -> Self {
        Self { frames }
    }
}

impl<T: Send + Sync + Clone + Copy + Frame> ToSignal for Cycle<T> {
    type Settings = Settings<T>;
    type Signal = oddio::Cycle<T>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::Cycle::new(settings.frames)
    }
}
