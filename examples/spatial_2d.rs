use bevy::{
    prelude::{
        default, App, Assets, BuildChildren, Camera2dBundle, Color, Commands, Component, Deref,
        Handle, Query, Res, ResMut, SpatialBundle, StartupStage, Transform, Vec2, Vec3, With,
    },
    sprite::{Sprite, SpriteBundle},
    time::Time,
    DefaultPlugins,
};
use bevy_oddio::{
    builtins::sine::{self, Sine},
    output::spatial::SpatialAudioSink,
    Audio, AudioPlugin,
};
use mint::{Point3, Vector3};
use oddio::{Sample, Spatial, SpatialOptions};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(init_assets)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup)
        .add_system(change_velocity)
        .run();
}

#[derive(Component)]
struct Emitter;

#[derive(Deref)]
struct SineHandle(Handle<Sine>);

struct SineSink(Handle<SpatialAudioSink<Sine>>);

fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<Sine>>) {
    let handle = assets.add(Sine);
    commands.insert_resource(SineHandle(handle));
}

fn setup(mut commands: Commands, mut audio: ResMut<Audio<Sample, Sine>>, noise: Res<SineHandle>) {
    // Note is in A4.
    let handle = audio.play_spatial(
        noise.clone(),
        sine::Settings::new(0.0, 440.0),
        // FIXME: Refactor this on bevy 0.9
        SpatialOptions {
            position: Point3 {
                x: 0.0,
                y: 0.4,
                z: 0.0,
            },
            velocity: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 0.5,
        },
    );
    commands.insert_resource(SineSink(handle));

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_scale(Vec3::splat(100.0)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN,
                        custom_size: Some(Vec2::new(0.3, 0.3)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.4, 0.0),
                    ..default()
                })
                .insert(Emitter);
        });

    commands.spawn_bundle(Camera2dBundle::default());
}

fn change_velocity(
    sink: Res<SineSink>,
    mut sinks: ResMut<Assets<SpatialAudioSink<Sine>>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Emitter>>,
) {
    let mut emitter = query.single_mut();

    let normalized_time = time.seconds_since_startup().sin() as f32 * 5.0;
    let delta = time.delta_seconds();

    let sink = match sinks.get_mut(&sink.0) {
        Some(sink) => sink,
        None => return,
    };

    let prev_pos = emitter.translation;
    let position = Point3 {
        x: normalized_time,
        y: prev_pos.y,
        z: prev_pos.z,
    };
    let velocity = Vector3 {
        x: (prev_pos.x - position.x) / delta,
        y: 0.0,
        z: 0.0,
    };

    emitter.translation = Vec3::new(position.x, position.y, position.z);

    sink.control::<Spatial<_>, _>()
        .set_motion(position, velocity, false);
}
