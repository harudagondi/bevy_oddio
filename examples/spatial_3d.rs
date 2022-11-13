use bevy::{
    prelude::{
        default, shape, App, Assets, Camera3dBundle, Color, Commands, Component, Deref, Handle,
        Mesh, PbrBundle, PointLight, PointLightBundle, Query, Res, ResMut, Resource,
        StandardMaterial, StartupStage, Transform, Vec3, With,
    },
    time::Time,
    DefaultPlugins,
};
use bevy_oddio::{
    builtins::sine::{self, Sine},
    output::spatial::SpatialAudioSink,
    Audio, AudioPlugin,
};
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

#[derive(Resource, Deref)]
struct SineHandle(Handle<Sine>);
#[derive(Resource)]
struct SineSink(Handle<SpatialAudioSink<Sine>>);

fn init_assets(mut commands: Commands, mut assets: ResMut<Assets<Sine>>) {
    let handle = assets.add(Sine);
    commands.insert_resource(SineHandle(handle));
}

fn setup(
    mut commands: Commands,
    mut audio: ResMut<Audio<Sample, Sine>>,
    noise: Res<SineHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // Listener
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 0.2,
            ..default()
        })),
        material: materials.add(Color::GREEN.into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Emitter
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.2,
                ..default()
            })),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Emitter);

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn change_velocity(
    sink: Res<SineSink>,
    mut sinks: ResMut<Assets<SpatialAudioSink<Sine>>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Emitter>>,
) {
    let mut emitter = query.single_mut();

    let x = time.elapsed_seconds_wrapped().sin() as f32 * 3.0;
    let z = time.elapsed_seconds_wrapped().cos() as f32 * 3.0;
    let delta = time.delta_seconds();

    let sink = match sinks.get_mut(&sink.0) {
        Some(sink) => sink,
        None => return,
    };

    let prev_pos = emitter.translation;
    let position = Vec3::new(x, prev_pos.y, z);
    let velocity = Vec3::new(
        (prev_pos.x - position.x) / delta,
        0.0,
        (prev_pos.z - position.z) / delta,
    );

    emitter.translation = Vec3::new(position.x, position.y, position.z);

    sink.control::<Spatial<_>, _>()
        .set_motion(position.into(), velocity.into(), false);
}
