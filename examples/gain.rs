use bevy::{
    prelude::{
        App, Assets, Commands, Deref, Handle, IntoSystemConfig, Res, ResMut, Resource, StartupSet,
    },
    reflect::TypeUuid,
    time::Time,
    DefaultPlugins,
};
use bevy_oddio::{builtins::sine, output::AudioSink, Audio, AudioApp, AudioPlugin, ToSignal};
use oddio::Sample;

#[derive(TypeUuid)]
#[uuid = "54498976-f7db-4ee7-a2e6-5fee0fcadbfb"]
struct SineWithGain;

impl ToSignal for SineWithGain {
    type Settings = sine::Settings;
    type Signal = oddio::Gain<oddio::Sine>;

    fn to_signal(&self, settings: Self::Settings) -> Self::Signal {
        oddio::Gain::new(oddio::Sine::new(settings.phase, settings.frequency_hz))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin::new())
        .add_audio_source::<1, _, SineWithGain>()
        .add_startup_system(init_assets)
        .add_startup_system(play_sine_with_gain.in_base_set(StartupSet::PostStartup))
        .add_system(change_volume)
        .run();
}

#[derive(Resource, Deref)]
struct SineWithGainHandle(Handle<SineWithGain>);
#[derive(Resource)]
struct SineWithGainSink(Handle<AudioSink<SineWithGain>>);

fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<SineWithGain>>) {
    let handle = assets.add(SineWithGain);
    commands.insert_resource(SineWithGainHandle(handle));
}

fn play_sine_with_gain(
    mut commands: Commands,
    mut audio: ResMut<Audio<Sample, SineWithGain>>,
    sine_with_gain: Res<SineWithGainHandle>,
) {
    let handle = audio.play(sine_with_gain.clone(), sine::Settings::new(0.0, 440.0));
    commands.insert_resource(SineWithGainSink(handle));
}

fn change_volume(
    sink_handle: Res<SineWithGainSink>,
    mut sinks: ResMut<Assets<AudioSink<SineWithGain>>>,
    time: Res<Time>,
) {
    let Some(sink) = sinks.get_mut(&sink_handle.0) else { return };

    let factor = (time.elapsed_seconds_wrapped().sin() + 1.0) / 2.0;

    sink.control::<oddio::Gain<_>, _>()
        .set_amplitude_ratio(factor);
}
