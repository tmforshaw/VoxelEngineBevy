use std::f32::consts::PI;

use bevy::{
    core::TaskPoolThreadAssignmentPolicy,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_screen_diagnostics::{
    ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};

use chunk_loading::{ChunkLoader, ChunkLoaderPlugin};
use constants::{CHUNK_LOAD_DISTANCE, FLYCAM_SENSITIVITY, FLYCAM_SPEED, MAX_THREADS, MIN_THREADS};
use rendering::{ChunkMaterial, GlobalChunkMaterial, RenderingPlugin};
use world::WorldPlugin;

pub mod chunk;
pub mod chunk_from_middle;
pub mod chunk_loading;
pub mod chunk_mesh;
pub mod constants;
pub mod culled_mesher;
pub mod greedy_mesher;
pub mod lod;
pub mod positions;
pub mod rendering;
pub mod vertex;
pub mod voxel;
pub mod world;

fn setup(mut commands: Commands, mut chunk_materials: ResMut<Assets<ChunkMaterial>>) {
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

    // Chunk shader material
    commands.insert_resource(GlobalChunkMaterial(chunk_materials.add(ChunkMaterial {
        reflectance: 0.5,
        perceptual_roughness: 0.5,
        metallic: 0.5,
    })))
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Ooga Booga Cube"),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions {
                        async_compute: TaskPoolThreadAssignmentPolicy {
                            min_threads: MIN_THREADS,
                            max_threads: MAX_THREADS,
                            percent: 0.75,
                        },
                        ..default()
                    },
                }),
        )
        .add_plugins((ChunkLoaderPlugin, WorldPlugin, RenderingPlugin))
        .add_plugins(NoCameraPlayerPlugin)
        // .add_plugins(WorldInspectorPlugin::new())
        // .add_plugins(AssetInspectorPlugin::<Mesh>::default())
        .add_plugins((
            ScreenDiagnosticsPlugin::default(),
            ScreenFrameDiagnosticsPlugin,
            ScreenEntityDiagnosticsPlugin,
        ))
        .insert_resource(MovementSettings {
            sensitivity: FLYCAM_SENSITIVITY,
            speed: FLYCAM_SPEED,
        })
        .insert_resource(KeyBindings {
            move_descend: KeyCode::ControlLeft,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}
