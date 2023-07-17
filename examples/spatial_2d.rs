use {
    bevy::{
        prelude::{
            default, App, Assets, BuildChildren, Camera2dBundle, Color, Commands, Component, Deref,
            Handle, PostStartup, Query, Res, ResMut, Resource, SpatialBundle, Startup, Transform,
            Update, Vec2, Vec3, With,
        },
        sprite::{Sprite, SpriteBundle},
        time::Time,
        DefaultPlugins,
    },
    bevy_oddio::{
        builtins::sine::{self, Sine},
        output::spatial::SpatialAudioSink,
        Audio, AudioPlugin,
    },
    oddio::{Sample, Spatial, SpatialOptions},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin::new())
        .add_systems(Startup, init_assets)
        .add_systems(PostStartup, setup)
        .add_systems(Update, change_velocity)
        .run();
}

#[derive(Component)]
struct Emitter;

#[derive(Resource, Deref)]
struct SineHandle(Handle<Sine>);
#[derive(Resource)]
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
        SpatialOptions {
            position: (Vec3::Y * 0.4).into(),
            velocity: Vec3::ZERO.into(),
            radius: 0.5,
        },
    );
    commands.insert_resource(SineSink(handle));

    commands
        .spawn(SpatialBundle {
            transform: Transform::from_scale(Vec3::splat(100.0)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
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

    commands.spawn(Camera2dBundle::default());
}

fn change_velocity(
    sink: Res<SineSink>,
    mut sinks: ResMut<Assets<SpatialAudioSink<Sine>>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Emitter>>,
) {
    let mut emitter = query.single_mut();

    let normalized_time = time.elapsed_seconds_wrapped().sin() * 5.0;
    let delta = time.delta_seconds();

    let Some(sink) = sinks.get_mut(&sink.0) else {
        return;
    };

    let prev_pos = emitter.translation;
    let position = Vec3::new(normalized_time, prev_pos.y, prev_pos.z);
    let velocity = Vec3::X * ((prev_pos.x - position.x) / delta);

    emitter.translation = Vec3::new(position.x, position.y, position.z);

    sink.control::<Spatial<_>, _>()
        .set_motion(position.into(), velocity.into(), false);
}
