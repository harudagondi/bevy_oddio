use bevy::{
    prelude::{App, Assets, Commands, Deref, Handle, Res, ResMut, StartupStage},
    DefaultPlugins,
};
use bevy_oddio::{
    builtins::sine::{self, Sine},
    output::AudioSink,
    Audio, AudioPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(add_to_assets)
        .add_startup_system_to_stage(StartupStage::PostStartup, play_sine_wave)
        .run();
}

#[derive(Deref)]
struct SineHandle(Handle<Sine>);

#[derive(Deref)]
struct SineAudioSink(Handle<AudioSink<Sine>>);

fn add_to_assets(mut commands: Commands, mut assets: ResMut<Assets<Sine>>) {
    let handle = assets.add(Sine);
    commands.insert_resource(SineHandle(handle));
}

fn play_sine_wave(mut commands: Commands, mut audio: ResMut<Audio<Sine>>, sine: Res<SineHandle>) {
    let handle = audio.play(sine.clone(), sine::Settings::new(0.0, 440.0));
    commands.insert_resource(SineAudioSink(handle))
}
