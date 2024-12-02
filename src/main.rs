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
use octree::{NodeDataType, Octree};
use rendering::{ChunkMaterial, GlobalChunkMaterial, RenderingPlugin};
use voxel::VoxelType;
use world::WorldPlugin;

pub mod chunk;
pub mod chunk_from_middle;
pub mod chunk_loading;
pub mod chunk_mesh;
pub mod constants;
pub mod culled_mesher;
pub mod greedy_mesher;
pub mod lod;
pub mod octree;
pub mod positions;
pub mod rendering;
pub mod serialise;
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
    })));
}

fn main() {
    let mut oct = Octree::new();

    // TODO adding at index 0 (position [0,0,0]) first causes data to disappear

    for x in (0..=1).rev() {
        for y in 0..=1 {
            for z in 0..=1 {
                oct.insert(
                    IVec3::new(x, y, z),
                    NodeDataType::new(VoxelType::Block, Color::linear_rgb(1., 1., 0.5)),
                );
            }
        }
    }

    // oct.insert(
    //     IVec3::new(0, 2, 0),
    //     NodeDataType::new(VoxelType::Block, Color::linear_rgb(0., 0., 0.)),
    // );
    // oct.insert(
    //     IVec3::new(1, 0, 0),
    //     NodeDataType::new(VoxelType::Block, Color::linear_rgb(0., 0., 0.)),
    // );
    // oct.insert(
    //     IVec3::new(0, 0, 1),
    //     NodeDataType::new(VoxelType::Block, Color::linear_rgb(0., 0., 0.)),
    // );

    // oct.insert(
    //     IVec3::new(0, 1, 0),
    //     NodeDataType::new(VoxelType::Block, Color::linear_rgb(0., 0., 0.)),
    // );

    // oct.insert(
    //     IVec3::new(0, 1, 1),
    //     NodeDataType::new(Color::linear_rgb(1., 1., 1.)),
    // );

    // println!("{:X?}", oct.depth_first(None));

    // oct.save_to_file("test.dat").unwrap();

    // println!("{:?}", oct.serialise());

    // let new_oct = Octree::load_from_file("test.dat").unwrap();
    // println!("{:?}", new_oct.depth_first(None));

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
        // .add_plugins((ChunkLoaderPlugin, WorldPlugin))
        .add_plugins(RenderingPlugin)
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
        .insert_resource(oct)
        .add_systems(Startup, (setup, Octree::draw_octree))
        // .add_systems(Startup, setup)
        .run();
}
