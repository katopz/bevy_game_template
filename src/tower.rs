use bevy::{prelude::*, utils::FloatOrd};
use bevy_mod_picking::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    assets::GameAssets,
    bullet::{Bullet, Lifetime},
    enemy::Target,
    GameState,
};

pub struct TowerPlugin;

#[derive(Component)]
pub struct Tower {
    pub shooting_timer: Timer,
    pub bullet_offset: Vec3,
    pub range: f32,
}

#[derive(Component)]
pub struct TowerBase {}

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_base)
            .add_event::<BuildTower>()
            .add_systems(Update, spawn_turret.run_if(on_event::<BuildTower>()))
            .add_systems(Update, tower_shooting.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_base(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("spawn tower base");
    let translation = Vec3::new(0.0, 2.0, -2.0);
    let transform = Transform::from_translation(translation);
    commands.spawn((
        TowerBase {},
        Name::new("tower_base"),
        SceneBundle {
            scene: asset_server.load("models/base.glb#Scene0"),
            transform,
            ..default()
        },
        Collider::cuboid(5.0, 5.0, 5.0),
        PickableBundle::default(),
        On::<Pointer<Click>>::run(spawn_turret),
    ));
}

// BUILD ===========

#[derive(Clone)]
enum TowerTurretType {
    Missile = 0,
}

#[derive(Clone, Event)]
pub struct BuildTower(TowerTurretType, Transform);

impl From<ListenerInput<Pointer<Click>>> for BuildTower {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        print!("event.target:{:?}", event.target);
        let translation = Vec3::new(0.0, 2.0, -2.0);
        let transform = Transform::from_translation(translation);

        BuildTower(TowerTurretType::Missile, transform)
    }
}

pub fn spawn_turret(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    event: Listener<Pointer<Click>>,
) {
    info!("🦀 spawn_turret: {:#?}", event.target);
    let translation = Vec3::new(0.0, 2.0, -2.0);
    let transform = Transform::from_translation(translation);

    commands.spawn((
        Tower {
            shooting_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            bullet_offset: translation,
            range: 100.0,
        },
        Name::new("tower_turret"),
        SceneBundle {
            scene: asset_server.load("models/turret_0.glb#Scene0"),
            transform,
            ..default()
        },
        Collider::cuboid(5.0, 5.0, 5.0),
        PickableBundle::default(),
        // On::<Pointer<Click>>::send_event::<BuildTower>(),
        // On::<Pointer<Click>>::send_event::<Shutdown>(),
    ));
}

// HUNT ===========

fn tower_shooting(
    mut commands: Commands,
    mut towers: Query<(Entity, &mut Tower, &GlobalTransform)>,
    targets: Query<&GlobalTransform, With<Target>>,
    game_assets: Res<GameAssets>,
    time: Res<Time>,
) {
    for (tower_ent, mut tower, transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let bullet_spawn = transform.translation() + tower.bullet_offset;

            let direction = targets
                .iter()
                .filter(|target_transform| {
                    Vec3::distance(target_transform.translation(), bullet_spawn) < tower.range
                })
                .min_by_key(|target_transform| {
                    FloatOrd(Vec3::distance(target_transform.translation(), bullet_spawn))
                })
                .map(|closest_target| closest_target.translation() - bullet_spawn);

            if let Some(direction) = direction {
                let bullet = Bullet {
                    direction,
                    speed: 5.0,
                };

                let translation = tower.bullet_offset;
                let transform = Transform::from_translation(translation);

                commands.entity(tower_ent).with_children(|commands| {
                    println!("tower_shooting");
                    commands
                        .spawn(SceneBundle {
                            scene: game_assets.bullet_scene.clone(),
                            transform,
                            ..Default::default()
                        })
                        .insert(Lifetime {
                            timer: Timer::from_seconds(10.0, TimerMode::Once),
                        })
                        .insert(bullet)
                        .insert(Name::new("Bullet"));
                });
            }
        }
    }
}
