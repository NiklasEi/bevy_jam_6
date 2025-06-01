use bevy::prelude::*;

use crate::{
    actions::{MoveDirection, NextMove, Orientation},
    GameState,
};

const ANIMATION_FRAMES: usize = 9;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct MovementTimer(pub Timer);

fn player_movement(
    time: Res<Time>,
    player: Query<(
        &mut MovementTimer,
        &mut Sprite,
        &mut Transform,
        &mut NextMove,
        &mut Orientation,
    )>,
) {
    for (mut timer, mut sprite, mut transform, mut next_move, mut orientation) in player {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                let row = atlas.index / ANIMATION_FRAMES;
                if 0 == (atlas.index + 1) % ANIMATION_FRAMES {
                    orientation.next(&next_move);
                    info!("moving towards {:?}", orientation.direction());
                    transform.translation += orientation.direction() * 64.;
                    transform.rotate_z(next_move.z_angle());
                    next_move.0 = MoveDirection::Straight;
                }
                atlas.index = row * ANIMATION_FRAMES + (atlas.index + 1) % ANIMATION_FRAMES
            }
        }
    }
}
