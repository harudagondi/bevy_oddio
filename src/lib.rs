#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! A plugin that integrates [`oddio`] with [`bevy`].

use std::sync::Arc;

use bevy::{
    prelude::{AddAsset, FromWorld, Plugin, World},
    reflect::TypeUuid,
    tasks::AsyncComputeTaskPool,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleRate,
};
use oddio::{Frames, FramesSignal, Handle, Mixer, Sample, Signal, SplitSignal, Stop};

pub use oddio;

mod loader;

type Stereo = [Sample; 2];

/// Resource that can play any type that implements [`Signal`].
pub struct Audio {
    mixer_handle: Handle<Mixer<Stereo>>,
    sample_rate: u32,
}

impl Audio {
    /// Play the given type that implements [`Signal`].
    ///
    /// Returns a handle that can be paused or permanently stopped.
    pub fn play<S>(&mut self, signal: S) -> Handle<Stop<S>>
    where
        S: Signal<Frame = Stereo> + Send + 'static,
    {
        self.mixer_handle.control().play(signal)
    }

    /// Returns the sample rate of the default device.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl FromWorld for Audio {
    fn from_world(world: &mut World) -> Self {
        let task_pool = world.resource::<AsyncComputeTaskPool>();
        let (mixer_handle, mixer) = oddio::split(oddio::Mixer::new());

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device available.");
        let sample_rate = device
            .default_output_config()
            .expect("Cannot get default output config.")
            .sample_rate();

        task_pool.scope(|scope| scope.spawn(play(mixer, device, sample_rate)));

        Self {
            mixer_handle,
            sample_rate: sample_rate.0,
        }
    }
}

#[allow(clippy::unused_async)]
async fn play(mixer: SplitSignal<Mixer<Stereo>>, device: Device, sample_rate: SampleRate) {
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };
    let stream = device
        .build_output_stream(
            &config,
            move |out_flat: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let out_stereo: &mut [Stereo] = oddio::frame_stereo(out_flat);
                oddio::run(&mixer, sample_rate.0, out_stereo);
            },
            move |err| bevy::utils::tracing::error!("Error in cpal: {err:?}"),
        )
        .expect("Cannot build output stream.");
    stream.play().expect("Cannot play stream.");

    // Do not drop the stream
    std::thread::sleep(std::time::Duration::MAX);
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

impl AudioSource {
    /// Convert the audio source to a [`FrameSignal`] that can be played using [`Audio`].
    #[must_use]
    pub fn to_signal(&self, start_seconds: f64) -> FramesSignal<Stereo> {
        FramesSignal::new(self.frames.clone(), start_seconds)
    }
}

/// Adds support for audio playback in a Bevy application.
///
/// Add this plugin to your Bevy app to get access to the [`Audio`] resource.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Audio>();
        app.add_asset::<AudioSource>();
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
