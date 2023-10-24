#![allow(clippy::type_complexity)]

mod actions;
mod assets;
mod audio;
mod bullet;
mod enemy;
mod ground;
mod loading;
mod menu;
mod player;
mod tower;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::enemy::EnemyPlugin;
use crate::ground::*;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::tower::TowerPlugin;

use assets::GameAssets;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use bevy_mod_picking::DefaultPickingPlugins;
// Game
use bevy_rapier3d::prelude::{Collider, NoUserData, RapierConfiguration, RapierPhysicsPlugin};
use bullet::BulletPlugin;
use oxidized_navigation::{
    debug_draw::{DrawNavMesh, DrawPath, OxidizedNavigationDebugDrawPlugin},
    query::{find_path, find_polygon_path, perform_string_pulling_on_path},
    tiles::NavMeshTiles,
    NavMesh, NavMeshAffector, NavMeshSettings, OxidizedNavigationPlugin,
};

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // Here the menu is drawn and waiting for player interaction
    Menu,
    // During this State the actual game logic is executed
    Playing,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                // MenuPlugin,
                // ActionsPlugin,
                // InternalAudioPlugin,
                PlayerPlugin,
                //
                DefaultPickingPlugins.build(),
                OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings {
                    cell_width: 0.25,
                    cell_height: 0.1,
                    tile_width: 100,
                    world_half_extents: 250.0,
                    world_bottom_bound: -100.0,
                    max_traversable_slope_radians: (40.0_f32 - 0.1).to_radians(),
                    walkable_height: 20,
                    walkable_radius: 1,
                    step_height: 3,
                    min_region_area: 100,
                    merge_region_area: 500,
                    max_contour_simplification_error: 1.1,
                    max_edge_length: 80,
                    max_tile_generation_tasks: Some(9),
                }),
                OxidizedNavigationDebugDrawPlugin,
                // The rapier plugin needs to be added for the scales of colliders to be correct if the scale of the entity is not uniformly 1.
                // An example of this is the "Thin Wall" in [setup_world_system]. If you remove this plugin, it will not appear correctly.
                RapierPhysicsPlugin::<NoUserData>::default(),
            ))
            .insert_resource(RapierConfiguration {
                physics_pipeline_active: false,
                ..Default::default()
            })
            .insert_resource(AsyncPathfindingTasks::default())
            .add_systems(Startup, (setup_world_system, info_system))
            .add_event::<DoSomethingComplex>()
            .add_systems(
                Update,
                (
                    run_blocking_pathfinding,
                    run_async_pathfinding,
                    poll_pathfinding_tasks_system,
                    draw_nav_mesh_system,
                    spawn_or_despawn_affector_system,
                    receive_greetings.run_if(on_event::<DoSomethingComplex>()),
                ),
            )
            .add_systems(PreStartup, asset_loading)
            .add_plugins(TowerPlugin)
            .add_plugins(EnemyPlugin)
            .add_plugins(BulletPlugin);

        #[cfg(debug_assertions)]
        {
            // app.add_plugins((LogDiagnosticsPlugin::default()));
        }
    }
}

// Assets =============

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    info!("asset_loading...");
    commands.insert_resource(GameAssets {
        enemy_scene: assets.load("models/enemy_0.glb#Scene0"),
        bullet_scene: assets.load("models/bullet.glb#Scene0"),
    });
}
