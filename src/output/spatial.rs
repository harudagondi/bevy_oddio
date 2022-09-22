use std::mem::ManuallyDrop;

use bevy::{
    asset::{Asset, Handle as BevyHandle, HandleId},
    prelude::{Assets, Deref, DerefMut, Res, ResMut},
    reflect::TypeUuid,
    tasks::AsyncComputeTaskPool,
    utils::HashMap,
};
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, SampleRate,
};
use oddio::{
    Frame, Handle as OddioHandle, Sample, Seek, Signal, Spatial, SpatialOptions, SpatialScene,
    SplitSignal, Stop,
};

use crate::{Audio, AudioToPlay, ToSignal};

use super::get_host_info;

/// Used internally in handling spatial audio output.
pub struct SpatialAudioOutput {
    spatial_scene_handle: OddioHandle<SpatialScene>,
}

impl SpatialAudioOutput {
    fn play<S>(&mut self, signal: S::Signal, options: SpatialOptions) -> SpatialAudioSink<S>
    where
        S: ToSignal + Asset,
        S::Signal: Seek + Signal<Frame = Sample> + Send,
    {
        SpatialAudioSink(ManuallyDrop::new(
            self.spatial_scene_handle.control().play(signal, options),
        ))
    }
}

impl Default for SpatialAudioOutput {
    fn default() -> Self {
        let task_pool = AsyncComputeTaskPool::get();
        let (spatial_scene_handle, spatial_scene) = oddio::split(SpatialScene::new());

        let (device, sample_rate) = get_host_info();

        task_pool
            .spawn(async move { play(spatial_scene, &device, sample_rate) })
            .detach();

        Self {
            spatial_scene_handle,
        }
    }
}

fn play(spatial_scene: SplitSignal<SpatialScene>, device: &Device, sample_rate: SampleRate) {
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = device
        .build_output_stream(
            &config,
            move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let out_stereo = oddio::frame_stereo(out);
                oddio::run(&spatial_scene, sample_rate.0, out_stereo);
            },
            move |err| bevy::utils::tracing::error!("Error in cpal: {err:?}"),
        )
        .expect("Cannot build output stream.");
    stream.play().expect("Cannot play stream.");

    // Do not drop the stream! or else there will be no audio
    std::thread::sleep(std::time::Duration::MAX);
}

/// System to play queued spatial audio in [`Audio`].
pub fn play_queued_spatial_audio<const N: usize, F, Source>(
    mut audio_output: ResMut<SpatialAudioOutput>,
    audio: Res<Audio<Sample, Source>>,
    sources: Res<Assets<Source>>,
    mut sink_assets: ResMut<Assets<SpatialAudioSink<Source>>>,
    mut sinks: ResMut<SpatialAudioSinks<Source>>,
) where
    Source: ToSignal + Asset + Send,
    Source::Signal: Seek + Signal<Frame = Sample> + Send,
{
    let mut queue = audio.queue.write();
    let len = queue.len();
    let mut i = 0;
    while i < len {
        let config = queue.pop_front().unwrap(); // This should not panic
        if let Some(audio_source) = sources.get(&config.source_handle) {
            if let Some(spatial_options) = config.spatial_options {
                let sink = audio_output
                    .play::<Source>(audio_source.to_signal(config.settings), spatial_options);
                // Unlike bevy_audio, we should not drop this
                let sink_handle = sink_assets.set(config.stop_handle, sink);
                sinks.insert(sink_handle.id, sink_handle.clone());
            }
        } else {
            queue.push_back(config);
        }
        i += 1;
    }
}

/// Asset that controls the playback of the spatial sound.
#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "4b135d1c-68cb-4104-b5c5-4be8bea6c46c"]
pub struct SpatialAudioSink<Source: ToSignal + Asset>(
    ManuallyDrop<OddioHandle<Spatial<Stop<<Source as ToSignal>::Signal>>>>,
);

/// Storage of all spatial audio sinks.
#[derive(Deref, DerefMut)]
pub struct SpatialAudioSinks<Source: ToSignal + Asset>(
    HashMap<HandleId, BevyHandle<SpatialAudioSink<Source>>>,
);

impl<Source: ToSignal + Asset> Default for SpatialAudioSinks<Source> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<F, Source> Audio<F, Source>
where
    Source: ToSignal + Asset,
    Source::Signal: Seek + Signal<Frame = f32>,
    F: Frame,
{
    /// Play the given type that implements [`Signal`].
    ///
    /// The signal must implement [`oddio::Seek`]
    /// and its frame must be [`f32`].
    ///
    /// Returns a handle that can be paused or permanently stopped.
    pub fn play_spatial(
        &mut self,
        source_handle: BevyHandle<Source>,
        settings: Source::Settings,
        spatial_options: SpatialOptions,
    ) -> BevyHandle<SpatialAudioSink<Source>> {
        let stop_handle = HandleId::random::<SpatialAudioSink<Source>>();
        let audio_to_play = AudioToPlay {
            source_handle,
            stop_handle,
            settings,
            spatial_options: Some(spatial_options),
        };
        self.queue.write().push_back(audio_to_play);
        BevyHandle::<SpatialAudioSink<Source>>::weak(stop_handle)
    }
}
