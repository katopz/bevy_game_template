use std::sync::{Arc, RwLock};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_mod_picking::prelude::*;
use bevy_rapier3d::prelude::{Collider, NoUserData, RapierConfiguration, RapierPhysicsPlugin};
use futures_lite::future;
use oxidized_navigation::{
    debug_draw::{DrawNavMesh, DrawPath, OxidizedNavigationDebugDrawPlugin},
    query::{find_path, find_polygon_path, perform_string_pulling_on_path},
    tiles::NavMeshTiles,
    NavMesh, NavMeshAffector, NavMeshSettings, OxidizedNavigationPlugin,
};

//
//  Blocking Pathfinding.
//  Press B to run.
//
//  Running pathfinding in a system.
//
pub fn run_blocking_pathfinding(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    nav_mesh_settings: Res<NavMeshSettings>,
    nav_mesh: Res<NavMesh>,
) {
    if !keys.just_pressed(KeyCode::B) {
        return;
    }

    // Get the underlying nav_mesh.
    if let Ok(nav_mesh) = nav_mesh.get().read() {
        let start_pos = Vec3::new(5.0, 1.0, 5.0);
        let end_pos = Vec3::new(-15.0, 1.0, -15.0);

        // Run pathfinding to get a polygon path.
        match find_polygon_path(
            &nav_mesh,
            &nav_mesh_settings,
            start_pos,
            end_pos,
            None,
            Some(&[1.0, 0.5]),
        ) {
            Ok(path) => {
                info!("Path found (BLOCKING): {:?}", path);

                // Convert polygon path to a path of Vec3s.
                match perform_string_pulling_on_path(&nav_mesh, start_pos, end_pos, &path) {
                    Ok(string_path) => {
                        info!("String path (BLOCKING): {:?}", string_path);
                        commands.spawn(DrawPath {
                            timer: Some(Timer::from_seconds(4.0, TimerMode::Once)),
                            pulled_path: string_path,
                            color: Color::RED,
                        });
                    }
                    Err(error) => error!("Error with string path: {:?}", error),
                };
            }
            Err(error) => error!("Error with pathfinding: {:?}", error),
        }
    }
}

//
//  Async Pathfinding.
//  Press A to run.
//
//  Running pathfinding in a task without blocking the frame.
//  Also check out Bevy's async compute example.
//  https://github.com/bevyengine/bevy/blob/main/examples/async_tasks/async_compute.rs
//

// Holder resource for tasks.
#[derive(Default, Resource)]
pub struct AsyncPathfindingTasks {
    tasks: Vec<Task<Option<Vec<Vec3>>>>,
}

// Queue up pathfinding tasks.
pub fn run_async_pathfinding(
    keys: Res<Input<KeyCode>>,
    nav_mesh_settings: Res<NavMeshSettings>,
    nav_mesh: Res<NavMesh>,
    mut pathfinding_task: ResMut<AsyncPathfindingTasks>,
) {
    if !keys.just_pressed(KeyCode::A) {
        return;
    }

    let thread_pool = AsyncComputeTaskPool::get();

    let nav_mesh_lock = nav_mesh.get();
    let start_pos = Vec3::new(5.0, 1.0, 5.0);
    let end_pos = Vec3::new(-15.0, 1.0, -15.0);

    let task = thread_pool.spawn(async_path_find(
        nav_mesh_lock,
        nav_mesh_settings.clone(),
        start_pos,
        end_pos,
        None,
    ));

    pathfinding_task.tasks.push(task);
}

// Poll existing tasks.
pub fn poll_pathfinding_tasks_system(
    mut commands: Commands,
    mut pathfinding_task: ResMut<AsyncPathfindingTasks>,
) {
    // Go through and remove completed tasks.
    pathfinding_task.tasks.retain_mut(|task| {
        if let Some(string_path) = future::block_on(future::poll_once(task)).unwrap_or(None) {
            info!("Async path task finished with result: {:?}", string_path);
            commands.spawn(DrawPath {
                timer: Some(Timer::from_seconds(4.0, TimerMode::Once)),
                pulled_path: string_path.clone(),
                color: Color::BLUE,
            });

            false
        } else {
            true
        }
    });
}

/// Async wrapper function for path finding.
pub async fn async_path_find(
    nav_mesh_lock: Arc<RwLock<NavMeshTiles>>,
    nav_mesh_settings: NavMeshSettings,
    start_pos: Vec3,
    end_pos: Vec3,
    position_search_radius: Option<f32>,
) -> Option<Vec<Vec3>> {
    // Get the underlying nav_mesh.
    let Ok(nav_mesh) = nav_mesh_lock.read() else {
        return None;
    };

    // Run pathfinding to get a path.
    match find_path(
        &nav_mesh,
        &nav_mesh_settings,
        start_pos,
        end_pos,
        position_search_radius,
        Some(&[1.0, 0.5]),
    ) {
        Ok(path) => {
            info!("Found path (ASYNC): {:?}", path);
            return Some(path);
        }
        Err(error) => error!("Error with pathfinding: {:?}", error),
    }

    None
}

//
//  Draw Nav-mesh.
//  Press M to toggle drawing the navmesh.
//
pub fn draw_nav_mesh_system(keys: Res<Input<KeyCode>>, mut draw_nav_mesh: ResMut<DrawNavMesh>) {
    if keys.just_pressed(KeyCode::M) {
        draw_nav_mesh.0 = !draw_nav_mesh.0;
    }
}

pub fn setup_world_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(30.0, 25.0, 25.0)
                .looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
            ..default()
        },
        RapierPickCamera::default(), // <- Sets the camera to use for picking.
    ));

    let heightfield_heights = (0..(50 * 50))
        .map(|value| {
            let position = value / 50;

            (position as f32 / 10.0).sin() / 10.0
        })
        .collect();

    // Heightfield.
    commands.spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        Collider::heightfield(heightfield_heights, 50, 50, Vec3::new(50.0, 50.0, 50.0)),
        NavMeshAffector,
    ));

    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //         material: materials.add(Color::RED.into()),
    //         transform: Transform::from_xyz(0.0, 5.0, -5.0),
    //         ..Default::default()
    //     },
    //     Collider::cuboid(1.0, 1.0, 1.0),
    //     NavMeshAffector,
    //     PickableBundle::default(), // <- Makes the collider pickable.
    //     // RapierPickTarget::default(), // <- Needed for the rapier picking backend
    //     On::<Pointer<Click>>::run(|event: Listener<Pointer<Click>>| {
    //         info!("🦀 Clicked on entity {:?}", event.target);
    //     }),
    // ));
}

pub fn spawn_or_despawn_affector_system(
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawned_entity: Local<Option<Entity>>,
) {
    if !keys.just_pressed(KeyCode::X) {
        return;
    }

    if let Some(entity) = *spawned_entity {
        commands.entity(entity).despawn_recursive();
        *spawned_entity = None;
    } else {
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 5.0 })),
                    material: materials.add(Color::BLUE.into()),
                    transform: Transform::from_xyz(-5.0, 5.0, -5.0),
                    ..default()
                },
                Collider::cuboid(2.5, 2.5, 2.5),
                NavMeshAffector, // Only entities with a NavMeshAffector component will contribute to the nav-mesh.
                PickableBundle::default(), // <- Makes the collider pickable.
                                 // On::<Pointer<Click>>::send_event::<Shutdown>(),
            ))
            .id();

        *spawned_entity = Some(entity);
    }
}

pub fn info_system() {
    info!("=========================================");
    info!("| Press A to run ASYNC path finding.    |");
    info!("| Press B to run BLOCKING path finding. |");
    info!("| Press M to draw nav-mesh.             |");
    info!("| Press X to spawn or despawn red cube. |");
    info!("=========================================");
}

#[derive(Event)]
pub struct DoSomethingComplex(Entity, f32);

impl From<ListenerInput<Pointer<Down>>> for DoSomethingComplex {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        print!("event.target:{:?}", event.target);
        DoSomethingComplex(event.target, event.hit.depth)
    }
}

/// Unlike callback systems, this is a normal system that can be run in parallel with other systems.
pub fn receive_greetings(mut greetings: EventReader<DoSomethingComplex>) {
    print!("receive_greetings");
    for event in greetings.iter() {
        info!(
            "🦀 Hello {:?}, you are {:?} depth units away from the pointer",
            event.0, event.1
        );
    }
}
