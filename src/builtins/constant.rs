use std::marker::PhantomData;

use bevy::reflect::TypeUuid;

use crate::ToSignal;

/// [`Asset`](bevy::asset::Asset) form of [`Constant`](oddio::Constant)
#[derive(TypeUuid, Default)]
#[uuid = "6bcf912a-91d0-46d3-bd55-e81123bbc591"]
pub struct Constant<T> {
    _phantom: PhantomData<T>,
}

impl<T> Constant<T> {
    /// Generate new `Constant` source
    #[must_use]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Settings for initialization of [`Constant`] audio source.
/// See [`Constant::new`](oddio::Constant::new) for more information.
pub struct Settings<T> {
    frame: T,
}

impl<T> Settings<T> {
    /// Generate settings for [`Constant`].
    pub fn new(frame: T) -> Self {
        Self { frame }
    }
}

impl<T: Send + Sync + Clone> ToSignal for Constant<T> {
    type Settings = Settings<T>;
    type Signal = oddio::Constant<T>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::Constant::new(settings.frame)
    }
}
