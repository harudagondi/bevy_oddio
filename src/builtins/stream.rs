use std::marker::PhantomData;

use bevy::reflect::TypeUuid;
use oddio::Frame;

use crate::ToSignal;

/// [`Asset`](bevy::asset::Asset) form of [`Stream`](oddio::Stream)
#[derive(TypeUuid, Default)]
#[uuid = "f391d20f-7654-403a-b7c9-3f3c7991138a"]
pub struct Stream<T> {
    _phantom: PhantomData<T>,
}

impl<T> Stream<T> {
    /// Generate new `Stream` source
    #[must_use]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Settings for initialization of [`Stream`] audio source.
/// See [`Stream::new`](oddio::Stream::new) for more information.
pub struct Settings {
    rate: u32,
    size: usize,
}

impl Settings {
    /// Generate settings for [`Stream`].
    #[must_use]
    pub fn new(rate: u32, size: usize) -> Self {
        Self { rate, size }
    }
}

impl<T: Send + Sync + Clone + Copy + Frame> ToSignal for Stream<T> {
    type Settings = Settings;
    type Signal = oddio::Stream<T>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::Stream::new(settings.rate, settings.size)
    }
}
