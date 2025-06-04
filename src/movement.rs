use bevy::{
    platform::{collections::HashMap, hash::Hashed},
    prelude::*,
};

use crate::{
    actions::{MoveDirection, NextMove, Orientation},
    following::Trailing,
    grid::{wrap_translate, TILE_SIZE},
    player::{SnakeHead, SnakeTail},
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
    tail: Query<Entity, With<SnakeTail>>,
    mut player_piece: Query<(
        Entity,
        &mut MovementTimer,
        &mut Sprite,
        &mut Transform,
        &mut NextMove,
        &mut Orientation,
        Option<&Trailing>,
    )>,
) -> Result {
    fn update_snake_piece(
        time: &Time,
        piece: (
            &mut MovementTimer,
            &mut Sprite,
            &mut Transform,
            &mut NextMove,
            &mut Orientation,
        ),
        new_move_direction: MoveDirection,
    ) -> Result {
        let (timer, sprite, transform, next_move, orientation) = piece;
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                let mut row = atlas.index / ANIMATION_FRAMES;
                if 0 == (atlas.index + 1) % ANIMATION_FRAMES {
                    orientation.next(&next_move);
                    info!("moving towards {:?}", orientation.direction());
                    transform.translation += orientation.direction() * TILE_SIZE;
                    wrap_translate(&mut transform.translation);
                    transform.rotate_z(next_move.z_angle());
                    next_move.0 = new_move_direction;
                    row = if new_move_direction == MoveDirection::Straight {
                        0
                    } else {
                        1
                    };
                    if new_move_direction == MoveDirection::Right {
                        sprite.flip_x = true;
                    } else {
                        sprite.flip_x = false;
                    }
                }
                atlas.index = row * ANIMATION_FRAMES + (atlas.index + 1) % ANIMATION_FRAMES
            }
        }

        Ok(())
    }

    let mut directions = HashMap::new();
    player_piece
        .iter()
        .for_each(|(entity, _, _, _, next_move, _, _)| {
            directions.insert(entity, next_move.0);
        });

    let mut next_entity = Some(tail.single()?);

    while let Some(entity) = next_entity {
        let (_, mut timer, mut sprite, mut transform, mut next_move, mut orientation, trailing) =
            player_piece.get_mut(entity)?;
        let new_move_direction = if let Some(trailing) = trailing {
            next_entity = Some(trailing.0);
            *directions
                .get(&trailing.0)
                .expect("trailed entity has no next_move")
        } else {
            next_entity = None;
            MoveDirection::Straight
        };
        update_snake_piece(
            &time,
            (
                &mut timer,
                &mut sprite,
                &mut transform,
                &mut next_move,
                &mut orientation,
            ),
            new_move_direction,
        )?;
    }

    Ok(())
}
