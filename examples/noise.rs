use bevy::{
    prelude::{App, Assets, Commands, Deref, Handle, Res, ResMut, StartupStage},
    reflect::TypeUuid,
    DefaultPlugins,
};
use bevy_oddio::{
    output::{AudioHandle, AudioSink},
    Audio, AudioApp, AudioPlugin, Stereo, ToSignal,
};
use oddio::Signal;

#[derive(TypeUuid)]
#[uuid = "7cc24057-b499-4f7a-8f8a-e37dfa64be32"]
struct Noise;
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
        .add_plugin(AudioPlugin)
        .add_audio_source::<Noise>()
        .add_startup_system(init_assets)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_noise)
        .run();
}

#[derive(Deref)]
struct NoiseHandle(Handle<Noise>);

struct NoiseSink(Handle<AudioHandle<Noise>>, Handle<AudioSink<Noise>>);

fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<Noise>>) {
    let handle = assets.add(Noise);
    commands.insert_resource(NoiseHandle(handle));
}

fn play_noise(mut commands: Commands, mut audio: ResMut<Audio<Noise>>, noise: Res<NoiseHandle>) {
    let handles = audio.play(noise.clone(), ());
    commands.insert_resource(NoiseSink(handles.0, handles.1));
}
