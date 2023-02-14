use bevy::{
    prelude::{App, Assets, Commands, Deref, Handle, Res, ResMut, Resource, StartupStage},
    DefaultPlugins,
};
use bevy_oddio::{
    builtins::sine::{self, Sine},
    output::AudioSink,
    Audio, AudioPlugin,
};
use oddio::Sample;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin::new())
        .add_startup_system(init_assets)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_sine)
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
