use crate::{
    gems::GemType,
    grid::{position_to_transform, GRID_HEIGHT, GRID_WIDTH},
    loading::TextureAssets,
    player::{ActivePositions, GridPosition},
    AppSystems, GamePhase, GameState,
};
use bevy::prelude::*;
use bevy_rand::{global::GlobalEntropy, prelude::ChaCha8Rng};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), fill_board)
            .add_systems(
                Update,
                mark_exploding
                    .in_set(AppSystems::Match)
                    .run_if(in_state(GamePhase::Playing))
                    .run_if(resource_changed::<ActivePositions>),
            );
    }
}

#[derive(Component)]
struct Exploding;

fn mark_exploding(
    active: Res<ActivePositions>,
    board: Res<Board>,
    mut commands: Commands,
    exploding: Query<Entity, With<Exploding>>,
    asset: Res<TextureAssets>,
) {
    exploding
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());

    let exploding = board.find_matches(&active.0);
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            if exploding[x][y] {
                commands.spawn((
                    Exploding,
                    Transform::from_translation(
                        position_to_transform(&GridPosition { x, y }).extend(0.),
                    ),
                    Sprite::from_image(asset.collision.clone()),
                ));
            }
        }
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

    fn find_matches(&self, active_slots: &Vec<GridPosition>) -> [[bool; GRID_HEIGHT]; GRID_WIDTH] {
        let mut checked = [[false; GRID_HEIGHT]; GRID_WIDTH];
        let mut exploding = [[false; GRID_HEIGHT]; GRID_WIDTH];
        for slot in active_slots {
            if checked[slot.x][slot.y] {
                continue;
            }
            checked[slot.x][slot.y] = true;
            let gem_type = self.gems[slot.x][slot.y].gem_type.clone();

            let mut x_diff = 1;
            let mut y_diff = 1;
            let mut matched_slot = false;
            let mut matched_x_slot = false;
            let mut matched_x_plus = false;
            let mut matched_y_slot = false;
            let mut matched_y_plus = false;
            while in_bounds(slot.x + x_diff, slot.y)
                && self.gems[slot.x + x_diff][slot.y].gem_type == gem_type
            {
                match x_diff {
                    1 => matched_x_plus = true,
                    2 => {
                        matched_slot = true;
                        matched_x_slot = true;
                        exploding[slot.x + 1][slot.y] = true;
                        exploding[slot.x + 2][slot.y] = true;
                    }
                    i => exploding[slot.x + i][slot.y] = true,
                };
                x_diff += 1;
            }
            x_diff = 1;
            while slot.x >= x_diff
                && in_bounds(slot.x - x_diff, slot.y)
                && self.gems[slot.x - x_diff][slot.y].gem_type == gem_type
            {
                match x_diff {
                    1 => {
                        if matched_x_plus {
                            exploding[slot.x - 1][slot.y] = true;
                            if !matched_x_slot {
                                matched_slot = true;
                                exploding[slot.x + 1][slot.y] = true;
                            }
                        }
                    }
                    2 => {
                        matched_slot = true;
                        if !matched_x_plus {
                            exploding[slot.x - 1][slot.y] = true;
                        }
                        exploding[slot.x - 2][slot.y] = true;
                    }
                    i => exploding[slot.x - i][slot.y] = true,
                };
                x_diff += 1;
            }

            while in_bounds(slot.x, slot.y + y_diff)
                && self.gems[slot.x][slot.y + y_diff].gem_type == gem_type
            {
                match y_diff {
                    1 => matched_y_plus = true,
                    2 => {
                        matched_slot = true;
                        matched_y_slot = true;
                        exploding[slot.x][slot.y + 1] = true;
                        exploding[slot.x][slot.y + 2] = true;
                    }
                    i => exploding[slot.x][slot.y + i] = true,
                };
                y_diff += 1;
            }
            y_diff = 1;
            while slot.y >= y_diff
                && in_bounds(slot.x, slot.y - y_diff)
                && self.gems[slot.x][slot.y - y_diff].gem_type == gem_type
            {
                match y_diff {
                    1 => {
                        if matched_y_plus {
                            exploding[slot.x][slot.y - 1] = true;
                            if !matched_y_slot {
                                matched_slot = true;
                                exploding[slot.x][slot.y + 1] = true;
                            }
                        }
                    }
                    2 => {
                        matched_slot = true;
                        if !matched_y_plus {
                            exploding[slot.x][slot.y - 1] = true;
                        }
                        exploding[slot.x][slot.y - 2] = true;
                    }
                    i => exploding[slot.x][slot.y - i] = true,
                };
                y_diff += 1;
            }

            if matched_slot {
                exploding[slot.x][slot.y] = true;
            }
        }

        exploding
    }
}

fn in_bounds(x: usize, y: usize) -> bool {
    return x < GRID_WIDTH && y < GRID_HEIGHT;
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
