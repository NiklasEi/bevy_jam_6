use crate::{
    gems::GemType,
    grid::{GRID_HEIGHT, GRID_WIDTH},
    player::GridPosition,
    GameState,
};
use bevy::prelude::*;
use bevy_rand::{global::GlobalEntropy, prelude::ChaCha8Rng};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), fill_board);
    }
}

#[derive(Resource, Default)]
pub struct Board {
    pub gems: [[Gem; GRID_HEIGHT]; GRID_WIDTH],
}

impl Board {
    pub fn randomize_gems(&mut self, rng: &mut GlobalEntropy<ChaCha8Rng>) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                self.gems[x][y].gem_type = GemType::random(rng);
            }
        }
    }

    fn find_matches() -> Vec<GridPosition> {
        vec![]
    }
}

#[derive(Clone)]
pub struct Gem {
    pub gem_type: GemType,
    pub entity: Option<Entity>,
}

impl Default for Gem {
    fn default() -> Self {
        Gem {
            gem_type: GemType::One,
            entity: None,
        }
    }
}

pub fn fill_board(mut commands: Commands, mut rng: GlobalEntropy<ChaCha8Rng>) {
    let mut board = Board::default();
    board.randomize_gems(&mut rng);
    commands.insert_resource(board);
}
