#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]

//! A plugin that integrates [`oddio`] with [`bevy`].
//!
//! Note that audio must have two channels or it will not work.
//! Thus, non-wav files are more likely to break.
//!
//! See [`#1`](https://github.com/harudagondi/bevy_oddio/issues/1).

use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use bevy::{
    asset::{Asset, HandleId},
    prelude::{AddAsset, App, CoreStage, Handle as BevyHandle, Plugin},
    reflect::TypeUuid,
};
use frames::{FromFrame, Mono, Stereo};
use oddio::{Frame, Frames, FramesSignal, Sample, Signal};

pub use oddio;
use output::{play_queued_audio, AudioHandle, AudioHandles, AudioOutput, AudioSink, AudioSinks};
use parking_lot::RwLock;

/// [`oddio`] builtin types that can be directly used in [`Audio::play`].
pub mod builtins;
/// Newtypes for working around [bevyengine/bevy#5432](https://github.com/bevyengine/bevy/issues/5432)
pub mod frames;

pub use frames::*;

mod loader;
/// Audio output
pub mod output;

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
pub struct Audio<F, Source = AudioSource<F>>
where
    Source: ToSignal + Asset,
    F: Frame,
{
    queue: RwLock<VecDeque<AudioToPlay<Source>>>,
    _frame: PhantomData<fn() -> F>,
}

impl<F, Source> Audio<F, Source>
where
    Source: ToSignal + Asset,
    F: Frame,
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

impl<F, Source> Default for Audio<F, Source>
where
    Source: ToSignal + Asset,
    F: Frame,
{
    fn default() -> Self {
        Self {
            queue: RwLock::default(),
            _frame: PhantomData,
        }
    }
}

/// Source of audio data.
///
/// Accepts an atomically reference-counted [`Frames`] with two channels.
#[derive(Clone, TypeUuid)]
#[uuid = "2b024eb6-88f1-4001-b678-0446f2fab0f4"]
pub struct AudioSource<F: Frame> {
    /// Raw audio data. See [`Frames`].
    pub frames: Arc<Frames<F>>,
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

impl<F: Frame + Send + Sync + Copy> ToSignal for AudioSource<F> {
    type Settings = f64;
    type Signal = FramesSignal<F>;

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
        app.init_resource::<AudioOutput<1, Mono>>()
            .init_resource::<AudioOutput<1, Sample>>()
            .init_resource::<AudioOutput<2, Stereo>>()
            .add_audio_source::<1, Mono, AudioSource<Mono>>()
            .add_audio_source::<2, Stereo, AudioSource<Stereo>>()
            .add_audio_source::<1, Sample, builtins::sine::Sine>();
        // .add_audio_source::<builtins::spatial_scene::SpatialScene>();
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
    fn add_audio_source<const N: usize, F, Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = F> + Send,
        F: Frame + FromFrame<[Sample; N]> + 'static;
}

impl AudioApp for App {
    fn add_audio_source<const N: usize, F, Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = F> + Send,
        F: Frame + FromFrame<[Sample; N]> + 'static,
    {
        self.add_asset::<Source>()
            .add_asset::<AudioSink<Source>>()
            .add_asset::<AudioHandle<Source>>()
            .init_resource::<Audio<F, Source>>()
            .init_resource::<AudioSinks<Source>>()
            .init_resource::<AudioHandles<Source>>()
            .add_system_to_stage(CoreStage::PostUpdate, play_queued_audio::<N, F, Source>)
    }
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct DocTestsForReadMe; // Only used for testing code blocks in README.md
