#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]

//! A plugin that integrates [`oddio`] with [`bevy`].
//!
//! There is an issue with loading audio files.
//!
//! See [`#4`](https://github.com/harudagondi/bevy_oddio/issues/4).

use {
    bevy::{
        asset::{Asset, HandleId},
        prelude::{
            AddAsset, App, CoreSet, Handle as BevyHandle, IntoSystemConfig, Plugin, Resource,
        },
        reflect::TypeUuid,
    },
    cpal::SupportedStreamConfigRange,
    oddio::{Frame, Frames, FramesSignal, Gain, Sample, Seek, Signal, SpatialOptions, Speed},
    output::{
        play_queued_audio,
        spatial::{
            play_queued_spatial_audio, play_queued_spatial_buffered_audio, SpatialAudioOutput,
            SpatialAudioSink, SpatialAudioSinks, SpatialBufferedAudioSink,
            SpatialBufferedAudioSinks,
        },
        AudioOutput, AudioSink, AudioSinks,
    },
    parking_lot::RwLock,
    std::{
        collections::VecDeque,
        marker::PhantomData,
        sync::{Arc, Mutex},
    },
};
pub use {cpal, oddio};

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
    settings: Source::Settings,
    spatial_settings: Option<SpatialSettings>,
}

struct SpatialSettings {
    options: SpatialOptions,
    buffered_settings: Option<BufferedSettings>,
}

struct BufferedSettings {
    max_distance: f32,
    rate: u32,
    buffer_duration: f32,
}

/// Resource that can play any type that implements [`Signal`].
#[derive(Resource)]
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
    ) -> BevyHandle<AudioSink<Source>> {
        let stop_handle = HandleId::random::<AudioSink<Source>>();
        let audio_to_play = AudioToPlay {
            source_handle,
            stop_handle,
            settings,
            spatial_settings: None,
        };
        self.queue.write().push_back(audio_to_play);
        BevyHandle::<AudioSink<Source>>::weak(stop_handle)
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
    type Signal = Gain<Speed<FramesSignal<F>>>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        Gain::new(Speed::new(FramesSignal::new(self.frames.clone(), settings)))
    }
}

#[derive(Resource)]
struct StreamConfig(SupportedStreamConfigRange);

/// Adds support for audio playback in a Bevy application.
///
/// Add this plugin to your Bevy app to get access to the [`Audio`] resource.
#[derive(Default)]
pub struct AudioPlugin {
    stream_config: Mutex<Option<SupportedStreamConfigRange>>,
}

impl AudioPlugin {
    /// Construct a default `AudioPlugin` without any specified stream configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an `AudioPlugin` with the specified stream configuration.
    #[must_use]
    pub fn with_stream_config(stream_config: SupportedStreamConfigRange) -> Self {
        Self {
            stream_config: Mutex::new(Some(stream_config)),
        }
    }
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioOutput<[Sample; 1]>>()
            .init_resource::<AudioOutput<Sample>>()
            .init_resource::<AudioOutput<[Sample; 2]>>()
            .add_audio_source::<[Sample; 1], AudioSource<[Sample; 1]>>()
            .add_audio_source::<[Sample; 2], AudioSource<[Sample; 2]>>()
            .add_audio_source::<Sample, builtins::sine::Sine>()
            .init_resource::<SpatialAudioOutput>()
            .add_spatial_audio_source::<builtins::sine::Sine>();

        if let Some(stream_config) = self.stream_config.lock().unwrap().take() {
            app.insert_resource(StreamConfig(stream_config));
        }

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
    fn add_audio_source<F, Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = F> + Send,
        F: Frame + 'static;

    /// Add support for custom spatial audio sources.
    ///
    /// There are two requirements the signal must meet:
    ///
    /// 1. Its frame must be [`Sample`]. Not `[Sample; 1]`.
    /// 2. It must implement [`Seek`].
    ///
    /// See [`SpatialSceneControl::play`].
    ///
    /// [`SpatialSceneControl::play`]: oddio::SpatialSceneControl::play
    fn add_spatial_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Sample> + Seek + Send;

    /// Add support for custom spatial buffered audio sources.
    ///
    /// There is one requirement the signal must meet:
    ///
    /// 1. Its frame must be [`Sample`]. Not `[Sample; 1]`.
    ///
    /// See [`SpatialSceneControl::play_buffered`].
    ///
    /// [`SpatialSceneControl::play_buffered`]: oddio::SpatialSceneControl::play_buffered
    fn add_spatial_buffered_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Sample> + Send;
}

/// Only one of these methods should be called for a given Source. Otherwise,
/// the wrong Audio system may deque a sound and skip playing it.
impl AudioApp for App {
    fn add_audio_source<F, Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = F> + Send,
        F: Frame + 'static,
    {
        self.add_asset::<Source>()
            .add_asset::<AudioSink<Source>>()
            .init_resource::<Audio<F, Source>>()
            .init_resource::<AudioSinks<Source>>()
            .add_system(play_queued_audio::<F, Source>.in_base_set(CoreSet::PostUpdate))
    }

    fn add_spatial_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Sample> + Seek + Send,
    {
        self.add_asset::<Source>()
            .add_asset::<SpatialAudioSink<Source>>()
            .init_resource::<Audio<Sample, Source>>()
            .init_resource::<SpatialAudioSinks<Source>>()
            .add_system(play_queued_spatial_audio::<Source>.in_base_set(CoreSet::PostUpdate))
    }

    fn add_spatial_buffered_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Sample> + Send,
    {
        self.add_asset::<Source>()
            .add_asset::<SpatialBufferedAudioSink<Source>>()
            .init_resource::<Audio<Sample, Source>>()
            .init_resource::<SpatialBufferedAudioSinks<Source>>()
            .add_system(
                play_queued_spatial_buffered_audio::<Source>.in_base_set(CoreSet::PostUpdate),
            )
    }
}

impl AudioApp for &mut App {
    fn add_audio_source<F, Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = F> + Send,
        F: Frame + 'static,
    {
        App::add_audio_source::<F, Source>(self);
        self
    }

    fn add_spatial_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Sample> + Seek + Send,
    {
        App::add_spatial_audio_source::<Source>(self);
        self
    }

    fn add_spatial_buffered_audio_source<Source>(&mut self) -> &mut Self
    where
        Source: ToSignal + Asset + Send,
        Source::Signal: Signal<Frame = Sample> + Send,
    {
        App::add_spatial_buffered_audio_source::<Source>(self);
        self
    }
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct DocTestsForReadMe; // Only used for testing code blocks in README.md
