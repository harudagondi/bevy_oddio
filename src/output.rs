use std::mem::ManuallyDrop;

use bevy::{
    asset::{Asset, Handle as BevyHandle, HandleId},
    prelude::{Assets, Deref, DerefMut, Res, ResMut},
    reflect::TypeUuid,
    tasks::AsyncComputeTaskPool,
    utils::HashMap,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SupportedBufferSize, SupportedStreamConfig,
};
use oddio::{Frame, Handle as OddioHandle, Mixer, Sample, Signal, SplitSignal, Stop};

use crate::{
    frames::{frame_n, FromFrame},
    Audio, ToSignal,
};

/// Used internally in handling audio output.
pub struct AudioOutput<const N: usize, F: Frame + FromFrame<[Sample; N]>> {
    mixer_handle: OddioHandle<Mixer<F>>,
}

impl<const N: usize, F: Frame + FromFrame<[Sample; N]> + 'static> AudioOutput<N, F> {
    fn play<S>(&mut self, signal: S::Signal) -> AudioSink<S>
    where
        S: ToSignal + Asset,
        S::Signal: Signal<Frame = F> + Send,
    {
        AudioSink(ManuallyDrop::new(self.mixer_handle.control().play(signal)))
    }
}

impl<const N: usize, F: Frame + FromFrame<[Sample; N]> + Clone + 'static> Default
    for AudioOutput<N, F>
{
    fn default() -> Self {
        let task_pool = AsyncComputeTaskPool::get();
        let (mixer_handle, mixer) = oddio::split(oddio::Mixer::new());

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device available.");
        let default_config = device
            .default_output_config()
            .expect("Cannot get default output config.");

        task_pool
            .spawn(play(mixer, device, default_config))
            .detach();

        Self { mixer_handle }
    }
}

#[allow(clippy::unused_async)]
async fn play<const N: usize, F>(
    mixer: SplitSignal<Mixer<F>>,
    device: Device,
    default_config: SupportedStreamConfig,
) where
    F: Frame + FromFrame<[Sample; N]> + 'static,
{
    assert!(
        N <= default_config.channels() as usize,
        "Device cannot support more than {N} channels!"
    );

    let buffer_size = match default_config.buffer_size() {
        SupportedBufferSize::Range { min, max: _ } => cpal::BufferSize::Fixed(*min),
        SupportedBufferSize::Unknown => cpal::BufferSize::Default,
    };
    let config = cpal::StreamConfig {
        channels: N.try_into().unwrap_or_else(|err| {
            panic!("Number of channels provided must be reasonable. Error: {err}")
        }),
        sample_rate: default_config.sample_rate(),
        buffer_size,
    };
    let stream = device
        .build_output_stream(
            &config,
            move |out_flat: &mut [f32], _: &cpal::OutputCallbackInfo| {
                assert_eq!(
                    out_flat.len() % N,
                    0,
                    "`N` must be a power of 2 that is less than or equal to the output buffer in cpal."
                );
                // Safety:
                // (1) `F` implements `FromFrame<[Sample; N]>`.
                // (2) out_flat.len() is divisible by `N`.
                let out_n = unsafe { frame_n(out_flat) };
                oddio::run(&mixer, config.sample_rate.0, out_n);
            },
            move |err| bevy::utils::tracing::error!("Error in cpal: {err:?}"),
        )
        .expect("Cannot build output stream.");
    stream.play().expect("Cannot play stream.");

    // Do not drop the stream! or else there will be no audio
    std::thread::sleep(std::time::Duration::MAX);
}

/// System to play queued audio in [`Audio`].
#[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
pub fn play_queued_audio<const N: usize, F, Source>(
    mut audio_output: ResMut<AudioOutput<N, F>>,
    audio: Res<Audio<F, Source>>,
    sources: Res<Assets<Source>>,
    mut sink_assets: ResMut<Assets<AudioSink<Source>>>,
    mut sinks: ResMut<AudioSinks<Source>>,
) where
    Source: ToSignal + Asset + Send,
    Source::Signal: Signal<Frame = F> + Send,
    F: Frame + FromFrame<[Sample; N]> + 'static,
{
    let mut queue = audio.queue.write();
    let len = queue.len();
    let mut i = 0;
    while i < len {
        let config = queue.pop_front().unwrap(); // This should not panic
        if let Some(audio_source) = sources.get(&config.source_handle) {
            let sink = audio_output.play::<Source>(audio_source.to_signal(config.settings));
            // Unlike bevy_audio, we should not drop this
            let sink_handle = sink_assets.set(config.stop_handle, sink);
            sinks.insert(sink_handle.id, sink_handle.clone());
        } else {
            queue.push_back(config);
        }
        i += 1;
    }
}

/// Asset that controls the playback of the sound.
#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "82317ee9-8f2d-4973-bb7f-8f4a5b74cc55"]
pub struct AudioSink<Source: ToSignal + Asset>(
    ManuallyDrop<OddioHandle<Stop<<Source as ToSignal>::Signal>>>,
);

/// Storage of all audio sinks.
#[derive(Deref, DerefMut)]
pub struct AudioSinks<Source: ToSignal + Asset>(HashMap<HandleId, BevyHandle<AudioSink<Source>>>);

impl<Source: ToSignal + Asset> Default for AudioSinks<Source> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}
