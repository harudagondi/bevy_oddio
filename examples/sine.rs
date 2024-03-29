use {
    bevy::{
        prelude::{
            App, Assets, Commands, Deref, Handle, PostStartup, Res, ResMut, Resource, Startup,
        },
        DefaultPlugins,
    },
    bevy_oddio::{
        builtins::sine::{self, Sine},
        output::AudioSink,
        Audio, AudioPlugin,
    },
    oddio::Sample,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin::new())
        .add_systems(Startup, init_assets)
        .add_systems(PostStartup, play_sine)
        .run();
}

#[derive(Resource, Deref)]
struct SineHandle(Handle<Sine>);
#[derive(Resource)]
struct SineSink(Handle<AudioSink<Sine>>);

fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<Sine>>) {
    let handle = assets.add(Sine);
    commands.insert_resource(SineHandle(handle));
}

fn play_sine(
    mut commands: Commands,
    mut audio: ResMut<Audio<Sample, Sine>>,
    noise: Res<SineHandle>,
) {
    // Note is in A4.
    let handle = audio.play(noise.clone(), sine::Settings::new(0.0, 440.0));
    commands.insert_resource(SineSink(handle));
}
