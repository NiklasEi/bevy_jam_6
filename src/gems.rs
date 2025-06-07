use bevy::prelude::*;
use bevy_rand::{global::GlobalEntropy, prelude::ChaCha8Rng};
use rand::Rng;

use crate::{
    board::{fill_board, Board},
    grid::{position_to_transform, GRID_HEIGHT, GRID_WIDTH},
    loading::TextureAssets,
    player::GridPosition,
    GameState,
};

pub struct GemsPlugin;

impl Plugin for GemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), draw_board.after(fill_board));
    }
}

fn draw_board(mut commands: Commands, assets: Res<TextureAssets>, board: Res<Board>) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let gem_type = board.gems[x][y].gem_type.clone();
            commands.spawn((
                Transform::from_translation(
                    position_to_transform(&GridPosition { x, y }).extend(0.),
                ),
                Sprite::from_image(assets.gem(&gem_type)),
                gem_type,
            ));
        }
    }
}

#[derive(PartialEq, Eq, Component, Clone)]
pub enum GemType {
    One,
    Two,
    Three,
    Four,
}

impl GemType {
    pub fn random(rng: &mut GlobalEntropy<ChaCha8Rng>) -> Self {
        match rng.gen_range(0..4) {
            0 => GemType::One,
            1 => GemType::Two,
            2 => GemType::Three,
            3 => GemType::Four,
            _ => unreachable!(),
        }
    }
}
