use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;

use crate::actions::{MoveDirection, NextMove, Orientation};

pub struct GridPlugin;

pub const GRID_WIDTH: usize = 16;
pub const GRID_HEIGHT: usize = 12;
pub const TILE_SIZE: f32 = 64.;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<ChaCha8Rng>::default());
    }
}

pub fn random_placement(
    length: u8,
    rng: &mut GlobalEntropy<ChaCha8Rng>,
) -> Vec<(Orientation, MoveDirection, Transform)> {
    let mut placements = vec![];

    let curves = vec![rng.gen_range(0..length - 1), rng.gen_range(0..length - 1)];

    let mut next_orientation = Orientation::Up;
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
        placements.push((next_orientation, direction, transform));

        next_rotation += NextMove(direction).z_angle();
        next_orientation.next(&NextMove(direction));
        next_position += next_orientation.direction() * TILE_SIZE;
    }

    placements
}
