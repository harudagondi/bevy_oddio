use bevy::{
    prelude::{App, Assets, Commands, Deref, Handle, Res, ResMut, Resource, StartupStage},
    reflect::TypeUuid,
    DefaultPlugins,
};
use bevy_oddio::{frames::Stereo, output::AudioSink, Audio, AudioApp, AudioPlugin, ToSignal};
use oddio::Signal;

#[derive(TypeUuid)]
#[uuid = "7cc24057-b499-4f7a-8f8a-e37dfa64be32"]
struct Noise;
#[derive(Resource)]
struct NoiseSignal;

impl Signal for NoiseSignal {
    type Frame = Stereo;

    fn sample(&self, _interval: f32, out: &mut [Self::Frame]) {
        for out_frame in out {
            let mono = fastrand::f32();
            out_frame[0] = mono;
            out_frame[1] = mono;
        }
    }
}

impl ToSignal for Noise {
    type Settings = ();
    type Signal = NoiseSignal;

    fn to_signal(&self, _settings: Self::Settings) -> Self::Signal {
        NoiseSignal
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin::new())
        .add_audio_source::<2, _, Noise>()
        .add_startup_system(init_assets)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_noise)
        .run();
}

#[derive(Resource, Deref)]
struct NoiseHandle(Handle<Noise>);
#[derive(Resource)]
struct NoiseSink(Handle<AudioSink<Noise>>);

fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<Noise>>) {
    let handle = assets.add(Noise);
    commands.insert_resource(NoiseHandle(handle));
}

fn play_noise(
    mut commands: Commands,
    mut audio: ResMut<Audio<Stereo, Noise>>,
    noise: Res<NoiseHandle>,
) {
    let handle = audio.play(noise.clone(), ());
    commands.insert_resource(NoiseSink(handle));
}
