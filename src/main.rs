use std::f32::consts::PI;

use bevy::{prelude::*, window::Cursor};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::{AssetInspectorPlugin, WorldInspectorPlugin};
use bevy_screen_diagnostics::{
    ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};

use chunk::Chunk;
use world::World;

pub mod chunk;
pub mod chunk_loading;
pub mod chunk_mesh;
pub mod culled_mesher;
pub mod positions;
pub mod vertex;
pub mod voxel;
pub mod world;

fn setup(mut commands: Commands) {
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 2., 0., 0.)),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(9.0, 9.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Ooga Booga Cube"),
                name: Some(String::from("Name of thing")),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NoCameraPlayerPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        // .add_plugins(AssetInspectorPlugin::<Mesh>::default())
        .add_plugins((
            ScreenDiagnosticsPlugin::default(),
            ScreenFrameDiagnosticsPlugin,
            ScreenEntityDiagnosticsPlugin,
        ))
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 16.0,          // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_descend: KeyCode::ControlLeft,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .insert_resource(World::new_with(vec![
            (0, 0, 0).into(),
            (1, 0, 0).into(),
            (2, 0, 0).into(),
            (3, 0, 0).into(),
            (0, 1, 0).into(),
            (0, 2, 0).into(),
            (0, 3, 0).into(),
        ]))
        // .add_systems(PostStartup, World::generate)
        .add_systems(PostStartup, culled_mesher::build_chunk_mesh)
        .run();
}
