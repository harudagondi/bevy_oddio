use {
    super::get_host_info,
    crate::{Audio, AudioToPlay, BufferedSettings, SpatialSettings, ToSignal},
    bevy::{
        asset::{Asset, Handle as BevyHandle, HandleId},
        prelude::{Assets, Deref, DerefMut, Quat, Res, ResMut, Resource},
        reflect::{TypePath, TypeUuid},
        tasks::AsyncComputeTaskPool,
        utils::HashMap,
    },
    cpal::{
        traits::{DeviceTrait, StreamTrait},
        Device, SupportedBufferSize, SupportedStreamConfigRange,
    },
    oddio::{
        Frame, Handle as OddioHandle, Sample, Seek, Signal, Spatial, SpatialBuffered,
        SpatialOptions, SpatialScene, SplitSignal, Stop,
    },
    std::mem::ManuallyDrop,
};

/// Used internally in handling spatial audio output.
#[derive(Resource)]
pub struct SpatialAudioOutput {
    spatial_scene_handle: OddioHandle<SpatialScene>,
}

impl SpatialAudioOutput {
    /// Rotate the listener.
    ///
    /// See [`SpatialSceneControl::set_listener_rotation`] for more information.
    ///
    /// [`SpatialSceneControl::set_listener_rotation`]: oddio::SpatialSceneControl::set_listener_rotation
    pub fn set_listener_rotation(&mut self, rotation: Quat) {
        self.spatial_scene_handle
            .control()
            .set_listener_rotation(rotation.into());
    }

    fn play<S>(&mut self, signal: S::Signal, options: SpatialOptions) -> SpatialAudioSink<S>
    where
        S: ToSignal + Asset,
        S::Signal: Seek + Signal<Frame = Sample> + Send,
    {
        SpatialAudioSink(ManuallyDrop::new(
            self.spatial_scene_handle.control().play(signal, options),
        ))
    }

    fn play_buffered<S>(
        &mut self,
        signal: S::Signal,
        options: SpatialOptions,
        max_distance: f32,
        rate: u32,
        buffer_duration: f32,
    ) -> SpatialBufferedAudioSink<S>
    where
        S: ToSignal + Asset,
        S::Signal: Signal<Frame = Sample> + Send,
    {
        SpatialBufferedAudioSink(ManuallyDrop::new(
            self.spatial_scene_handle.control().play_buffered(
                signal,
                options,
                max_distance,
                rate,
                buffer_duration,
            ),
        ))
    }
}

impl Default for SpatialAudioOutput {
    fn default() -> Self {
        let task_pool = AsyncComputeTaskPool::get();
        let (spatial_scene_handle, spatial_scene) = oddio::split(SpatialScene::new());

        let (device, supported_config_range) = get_host_info();

        task_pool
            .spawn(async move { play(spatial_scene, &device, &supported_config_range) })
            .detach();

        Self {
            spatial_scene_handle,
        }
    }
}

fn play(
    spatial_scene: SplitSignal<SpatialScene>,
    device: &Device,
    supported_config_range: &SupportedStreamConfigRange,
) {
    let buffer_size = match supported_config_range.buffer_size() {
        SupportedBufferSize::Range { min, max: _ } => cpal::BufferSize::Fixed(*min),
        SupportedBufferSize::Unknown => cpal::BufferSize::Default,
    };
    let config = cpal::StreamConfig {
        channels: supported_config_range.channels(),
        sample_rate: supported_config_range.max_sample_rate(),
        buffer_size,
    };
    let stream = device
        .build_output_stream(
            &config,
            move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let out_stereo = oddio::frame_stereo(out);
                oddio::run(&spatial_scene, config.sample_rate.0, out_stereo);
            },
            move |err| bevy::utils::tracing::error!("Error in cpal: {err:?}"),
            None,
        )
        .expect("Cannot build output stream.");
    stream.play().expect("Cannot play stream.");

    // Do not drop the stream! or else there will be no audio
    std::mem::forget(stream);
}

/// System to play queued spatial audio in [`Audio`].
#[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
pub fn play_queued_spatial_audio<Source>(
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
            if let Some(spatial_settings) = config.spatial_settings {
                let sink = audio_output.play::<Source>(
                    audio_source.to_signal(config.settings),
                    spatial_settings.options,
                );
                // Unlike bevy_audio, we should not drop this
                let sink_handle = sink_assets.set(config.stop_handle, sink);
                sinks.insert(sink_handle.id(), sink_handle.clone());
            }
        } else {
            queue.push_back(config);
        }
        i += 1;
    }
}

/// System to play queued spatial buffered audio in [`Audio`].
#[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
pub fn play_queued_spatial_buffered_audio<Source>(
    mut audio_output: ResMut<SpatialAudioOutput>,
    audio: Res<Audio<Sample, Source>>,
    sources: Res<Assets<Source>>,
    mut sink_assets: ResMut<Assets<SpatialBufferedAudioSink<Source>>>,
    mut sinks: ResMut<SpatialBufferedAudioSinks<Source>>,
) where
    Source: ToSignal + Asset + Send,
    Source::Signal: Signal<Frame = Sample> + Send,
{
    let mut queue = audio.queue.write();
    let len = queue.len();
    let mut i = 0;
    while i < len {
        let config = queue.pop_front().unwrap(); // This should not panic
        if let Some(audio_source) = sources.get(&config.source_handle) {
            if let Some(spatial_settings) = config.spatial_settings {
                if let Some(BufferedSettings {
                    max_distance,
                    rate,
                    buffer_duration,
                }) = spatial_settings.buffered_settings
                {
                    let sink = audio_output.play_buffered::<Source>(
                        audio_source.to_signal(config.settings),
                        spatial_settings.options,
                        max_distance,
                        rate,
                        buffer_duration,
                    );
                    // Unlike bevy_audio, we should not drop this
                    let sink_handle = sink_assets.set(config.stop_handle, sink);
                    sinks.insert(sink_handle.id(), sink_handle.clone());
                }
            }
        } else {
            queue.push_back(config);
        }
        i += 1;
    }
}

/// Asset that controls the playback of the spatial sound.
#[derive(TypeUuid, TypePath, Deref, DerefMut)]
#[uuid = "4b135d1c-68cb-4104-b5c5-4be8bea6c46c"]
pub struct SpatialAudioSink<Source: ToSignal + Asset>(
    ManuallyDrop<OddioHandle<Spatial<Stop<<Source as ToSignal>::Signal>>>>,
);

/// Storage of all spatial audio sinks.
#[derive(Resource, Deref, DerefMut)]
pub struct SpatialAudioSinks<Source: ToSignal + Asset>(
    HashMap<HandleId, BevyHandle<SpatialAudioSink<Source>>>,
);

impl<Source: ToSignal + Asset> Default for SpatialAudioSinks<Source> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

/// Asset that controls the playback of the spatial sound.
#[derive(TypeUuid, TypePath, Deref, DerefMut)]
#[uuid = "4b135d1c-68cb-4104-b5c5-4be8bea6c46c"]
pub struct SpatialBufferedAudioSink<Source: ToSignal + Asset>(
    ManuallyDrop<OddioHandle<SpatialBuffered<Stop<<Source as ToSignal>::Signal>>>>,
);

/// Storage of all spatial audio sinks.
#[derive(Resource, Deref, DerefMut)]
pub struct SpatialBufferedAudioSinks<Source: ToSignal + Asset>(
    HashMap<HandleId, BevyHandle<SpatialBufferedAudioSink<Source>>>,
);

impl<Source: ToSignal + Asset> Default for SpatialBufferedAudioSinks<Source> {
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
            spatial_settings: Some(SpatialSettings {
                options: spatial_options,
                buffered_settings: None,
            }),
        };
        self.queue.write().push_back(audio_to_play);
        BevyHandle::<SpatialAudioSink<Source>>::weak(stop_handle)
    }
}

impl<F, Source> Audio<F, Source>
where
    Source: ToSignal + Asset,
    Source::Signal: Signal<Frame = f32>,
    F: Frame,
{
    /// Play the given type that implements [`Signal`].
    ///
    /// The signal's frame must be [`f32`].
    ///
    /// Returns a handle that can be paused or permanently stopped.
    pub fn play_spatial_buffered(
        &mut self,
        source_handle: BevyHandle<Source>,
        settings: Source::Settings,
        spatial_options: SpatialOptions,
        max_distance: f32,
        rate: u32,
        buffer_duration: f32,
    ) -> BevyHandle<SpatialBufferedAudioSink<Source>> {
        let stop_handle = HandleId::random::<SpatialBufferedAudioSink<Source>>();
        let audio_to_play = AudioToPlay {
            source_handle,
            stop_handle,
            settings,
            spatial_settings: Some(SpatialSettings {
                options: spatial_options,
                buffered_settings: Some(BufferedSettings {
                    max_distance,
                    rate,
                    buffer_duration,
                }),
            }),
        };
        self.queue.write().push_back(audio_to_play);
        BevyHandle::<SpatialBufferedAudioSink<Source>>::weak(stop_handle)
    }
}
