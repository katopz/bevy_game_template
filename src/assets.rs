use bevy::prelude::*;

#[derive(Resource)]
pub struct GameAssets {
    pub enemy_scene: Handle<Scene>,
    pub bullet_scene: Handle<Scene>,
}
