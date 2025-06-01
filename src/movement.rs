use bevy::prelude::*;

use crate::GameState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct MovementTimer(pub Timer);

fn player_movement(time: Res<Time>, player: Query<(&mut MovementTimer, &mut Sprite)>) {
    for (mut timer, mut sprite) in player {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                let row = atlas.index / 9;
                atlas.index = row * 9 + (atlas.index + 1) % 9
            }
        }
    }
}
