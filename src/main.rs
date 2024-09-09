use std::f32::consts::PI;

use bevy::{core::TaskPoolThreadAssignmentPolicy, prelude::*};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_screen_diagnostics::{
    ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};

use chunk_loading::{ChunkLoader, ChunkLoaderPlugin, CHUNK_LOAD_DISTANCE};
use world::WorldPlugin;

pub mod chunk;
pub mod chunk_from_middle;
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
        ChunkLoader::new(CHUNK_LOAD_DISTANCE),
        Camera3dBundle {
            transform: Transform::from_xyz(9.0, 9.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Ooga Booga Cube"),
                        name: Some(String::from("Name of thing")),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions {
                        async_compute: TaskPoolThreadAssignmentPolicy {
                            min_threads: 1,
                            max_threads: 8,
                            percent: 0.75,
                        },
                        ..default()
                    },
                }),
        )
        .add_plugins(ChunkLoaderPlugin)
        .add_plugins(WorldPlugin)
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
            speed: 64.0,          // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_descend: KeyCode::ControlLeft,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}
