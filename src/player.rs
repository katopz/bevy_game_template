use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

// Could be a resource
#[derive(Component)]
pub struct Player {
    pub money: u32,
    pub health: u32,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        // .add_systems(Update, move_player.run_if(in_state(GameState::Playing)));
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Player {
            money: 100,
            health: 10,
        },
        Name::new("Player"),
    ));
}
