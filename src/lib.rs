#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! A plugin that integrates [`oddio`] with [`bevy`].
//!
//! Note that audio must have two channels or it will not work.
//! Thus, non-wav files are more likely to break.
//!
//! When implementing [`oddio::Signal`] for your types,
//! use [`Stereo`] as your output.
//!
//! See [`#1`](https://github.com/harudagondi/bevy_oddio/issues/1).

use std::{collections::VecDeque, sync::Arc};

use bevy::{
    asset::{Asset, HandleId},
    prelude::{AddAsset, App, CoreStage, Handle as BevyHandle, Plugin},
    reflect::TypeUuid,
};
use oddio::{Frames, FramesSignal, Sample, Signal};

pub use oddio;
use output::{play_queued_audio, AudioHandle, AudioHandles, AudioOutput, AudioSink, AudioSinks};
use parking_lot::RwLock;

/// [`oddio`] builtin types that can be directly used in [`Audio::play`].
pub mod builtins;
mod loader;
/// Audio output
pub mod output;

/// The frame used in the `oddio` types
pub type Stereo = [Sample; 2];

struct AudioToPlay<Source>
where
    Source: ToSignal + Asset,
{
    source_handle: BevyHandle<Source>,
    stop_handle: HandleId,
    audio_handle: HandleId,
    settings: Source::Settings,
}

/// Resource that can play any type that implements [`Signal`].
pub struct Audio<Source = AudioSource>
where
    Source: ToSignal + Asset,
{
    queue: RwLock<VecDeque<AudioToPlay<Source>>>,
}

impl<Source> Audio<Source>
where
    Source: ToSignal + Asset,
{
    /// Play the given type that implements [`Signal`].
    ///
    /// Returns a handle that can be paused or permanently stopped.
    pub fn play(
        &mut self,
        source_handle: BevyHandle<Source>,
        settings: Source::Settings,
    ) -> (
        BevyHandle<AudioHandle<Source>>,
        BevyHandle<AudioSink<Source>>,
    ) {
        let stop_handle = HandleId::random::<AudioSink<Source>>();
        let audio_handle = HandleId::random::<AudioHandle<Source>>();
        let audio_to_play = AudioToPlay {
            source_handle,
            stop_handle,
            audio_handle,
            settings,
        };
        self.queue.write().push_back(audio_to_play);
        (
            BevyHandle::<AudioHandle<Source>>::weak(audio_handle),
            BevyHandle::<AudioSink<Source>>::weak(stop_handle),
        )
    }
}

impl<Source> Default for Audio<Source>
where
    Source: ToSignal + Asset,
{
    fn default() -> Self {
        Self {
            queue: RwLock::default(),
        }
    }
}

/// Source of audio data.
///
/// Accepts an atomically reference-counted [`Frames`] with two channels.
#[derive(Clone, TypeUuid)]
#[uuid = "2b024eb6-88f1-4001-b678-0446f2fab0f4"]
pub struct AudioSource {
    /// Raw audio data. See [`Frames`].
    pub frames: Arc<Frames<Stereo>>,
}

/// Trait for a type that generates a signal.
pub trait ToSignal {
    /// The settings needed to initialize the signal.
    /// See [`oddio`]'s types and its `new` associated method
    /// for examples of a `Settings`.
    type Settings: Send + Sync;
    /// The [`Signal`](oddio::Signal) produced by the
    /// type implementing this trait.
    type Signal: Signal + Send;

    /// Create a new [`Signal`](oddio::Signal)
    /// based on the implementing type.
    fn to_signal(&self, settings: Self::Settings) -> Self::Signal;
}

impl ToSignal for AudioSource {
    type Settings = f64;
    type Signal = FramesSignal<Stereo>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        FramesSignal::new(self.frames.clone(), settings)
    }
}

/// Adds support for audio playback in a Bevy application.
///
/// Add this plugin to your Bevy app to get access to the [`Audio`] resource.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioOutput>()
            .add_audio_source::<AudioSource>()
            // .add_audio_source::<builtins::sine::Sine>()
            .add_audio_source::<builtins::spatial_scene::SpatialScene>();
        #[cfg(feature = "flac")]
        app.init_asset_loader::<loader::flac_loader::FlacLoader>();
        #[cfg(feature = "mp3")]
        app.init_asset_loader::<loader::mp3_loader::Mp3Loader>();
        #[cfg(feature = "ogg")]
        app.init_asset_loader::<loader::ogg_loader::OggLoader>();
        #[cfg(feature = "wav")]
        app.init_asset_loader::<loader::wav_loader::WavLoader>();
    }
}

/// Extension trait to add new audio sources implemented by users
pub trait AudioApp {
    /// Add support for custom audio sources.
    fn add_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Stereo> + Send;
}

impl AudioApp for App {
    fn add_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Stereo> + Send,
    {
        self.add_asset::<Source>()
            .add_asset::<AudioSink<Source>>()
            .add_asset::<AudioHandle<Source>>()
            .init_resource::<Audio<Source>>()
            .init_resource::<AudioSinks<Source>>()
            .init_resource::<AudioHandles<Source>>()
            .add_system_to_stage(CoreStage::PostUpdate, play_queued_audio::<Source>)
    }
}
