use {
    bevy::{
        prelude::{
            App, Assets, Commands, Deref, Handle, PostStartup, Res, ResMut, Resource, Startup,
        },
        reflect::{TypePath, TypeUuid},
        DefaultPlugins,
    },
    bevy_oddio::{output::AudioSink, Audio, AudioApp, AudioPlugin, ToSignal},
    oddio::{Sample, Signal},
};

#[derive(TypeUuid, TypePath)]
#[uuid = "7cc24057-b499-4f7a-8f8a-e37dfa64be32"]
struct Noise;
#[derive(Resource)]
struct NoiseSignal;

impl Signal for NoiseSignal {
    type Frame = [Sample; 2];

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
        .add_plugins(AudioPlugin::new())
        .add_audio_source::<_, Noise>()
        .add_systems(Startup, init_assets)
        .add_systems(PostStartup, play_noise)
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
    mut audio: ResMut<Audio<[Sample; 2], Noise>>,
    noise: Res<NoiseHandle>,
) {
    let handle = audio.play(noise.clone(), ());
    commands.insert_resource(NoiseSink(handle));
}
