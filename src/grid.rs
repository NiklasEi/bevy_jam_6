use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;

use crate::{
    actions::{MoveDirection, NextMove, Orientation},
    loading::TextureAssets,
    player::GridPosition,
    GameState,
};

pub struct GridPlugin;

pub const GRID_WIDTH: usize = 16;
pub const GRID_HEIGHT: usize = 10;
pub const TILE_SIZE: f32 = 64.;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
            .add_systems(OnEnter(GameState::Playing), spawn_grid);
    }
}

pub fn random_placement(
    length: u8,
    rng: &mut GlobalEntropy<ChaCha8Rng>,
) -> Vec<(Orientation, MoveDirection, Transform, GridPosition)> {
    let mut placements = vec![];

    let curves = [rng.gen_range(0..length - 1), rng.gen_range(0..length - 1)];

    let mut next_orientation = Orientation::Up;
    let mut next_grid_position = GridPosition {
        x: GRID_WIDTH / 2,
        y: GRID_HEIGHT / 2,
    };
    let mut next_position = Vec3::new(0., 0., 1.);
    let mut next_rotation = 0.;
    for i in 0..length {
        let direction = if curves.contains(&i) {
            if rng.gen_bool(0.5) {
                MoveDirection::Left
            } else {
                MoveDirection::Right
            }
        } else {
            MoveDirection::Straight
        };

        let mut transform = Transform::from_translation(next_position);
        transform.rotate_z(next_rotation);
        placements.push((
            next_orientation,
            direction,
            transform,
            next_grid_position.clone(),
        ));

        next_rotation += NextMove(direction).z_angle();
        next_orientation.next(&NextMove(direction));
        next_grid_position = next_orientation.next_position(&next_grid_position);
        next_position += next_orientation.direction() * TILE_SIZE;
        wrap_translate(&mut next_position);
    }

    placements
}

pub fn wrap_translate(translate: &mut Vec3) {
    if translate.x > (-(GRID_WIDTH as f32) / 2. + GRID_WIDTH as f32 - 0.5) * TILE_SIZE {
        translate.x -= GRID_WIDTH as f32 * TILE_SIZE;
    } else if translate.x < (-(GRID_WIDTH as f32) / 2. - 0.5) * TILE_SIZE {
        translate.x += GRID_WIDTH as f32 * TILE_SIZE;
    }
    if translate.y > (GRID_HEIGHT as f32 / 2. - 0.5) * TILE_SIZE {
        translate.y -= GRID_HEIGHT as f32 * TILE_SIZE;
    } else if translate.y < (GRID_HEIGHT as f32 / 2. - GRID_HEIGHT as f32 - 0.5) * TILE_SIZE {
        translate.y += GRID_HEIGHT as f32 * TILE_SIZE;
    }
}

pub fn position_to_transform(position: &GridPosition) -> Vec2 {
    Vec2::new(
        (-(GRID_WIDTH as f32) / 2. + position.x as f32) * TILE_SIZE,
        -(GRID_HEIGHT as f32 / 2. - position.y as f32) * TILE_SIZE,
    )
}

fn spawn_grid(mut commands: Commands, textures: Res<TextureAssets>) {
    for column in 0..GRID_WIDTH {
        for row in 1..=GRID_HEIGHT {
            commands.spawn((
                Sprite::from_image(textures.tile.clone()),
                Transform::from_translation(Vec3::new(
                    (-(GRID_WIDTH as f32) / 2. + column as f32) * TILE_SIZE,
                    (GRID_HEIGHT as f32 / 2. - row as f32) * TILE_SIZE,
                    0.,
                )),
            ));
        }
    }
}
