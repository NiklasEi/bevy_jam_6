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

    for explode in board.find_matches(&active.0) {
        commands.spawn((
            Exploding,
            Transform::from_translation(position_to_transform(&explode).extend(0.)),
            Sprite::from_image(asset.collision.clone()),
        ));
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

    fn find_matches(&self, active_slots: &Vec<GridPosition>) -> Vec<GridPosition> {
        let mut matched = vec![];
        for slot in active_slots {
            let gem_type = self.gems[slot.x][slot.y].gem_type.clone();
            let mut matched_slot = false;
            let mut matched_x_inner = false;
            let mut matched_x_outer = false;
            let mut matched_y_inner = false;
            let mut matched_y_outer = false;
            if slot.x > 0
                && in_bounds(slot.x - 1, slot.y)
                && self.gems[slot.x - 1][slot.y].gem_type == gem_type
            {
                matched_x_inner = true;
                if slot.x > 1
                    && in_bounds(slot.x - 2, slot.y)
                    && self.gems[slot.x - 2][slot.y].gem_type == gem_type
                {
                    matched.push(GridPosition {
                        x: slot.x - 1,
                        y: slot.y,
                    });
                    matched.push(GridPosition {
                        x: slot.x - 2,
                        y: slot.y,
                    });
                    matched_slot = true;
                    matched_x_outer = true;
                }
            }
            if in_bounds(slot.x + 1, slot.y) && self.gems[slot.x + 1][slot.y].gem_type == gem_type {
                if in_bounds(slot.x + 2, slot.y)
                    && self.gems[slot.x + 2][slot.y].gem_type == gem_type
                {
                    matched.push(GridPosition {
                        x: slot.x + 1,
                        y: slot.y,
                    });
                    matched.push(GridPosition {
                        x: slot.x + 2,
                        y: slot.y,
                    });
                    if matched_x_inner && !matched_x_outer {
                        matched.push(GridPosition {
                            x: slot.x - 1,
                            y: slot.y,
                        });
                    }
                    matched_slot = true;
                } else if matched_x_inner {
                    matched_slot = true;
                    matched.push(GridPosition {
                        x: slot.x + 1,
                        y: slot.y,
                    });
                    if !matched_x_outer {
                        matched.push(GridPosition {
                            x: slot.x - 1,
                            y: slot.y,
                        });
                    }
                }
            }

            if in_bounds(slot.x, slot.y + 1) && self.gems[slot.x][slot.y + 1].gem_type == gem_type {
                matched_y_inner = true;
                if in_bounds(slot.x, slot.y + 2)
                    && self.gems[slot.x][slot.y + 2].gem_type == gem_type
                {
                    matched.push(GridPosition {
                        x: slot.x,
                        y: slot.y + 1,
                    });
                    matched.push(GridPosition {
                        x: slot.x,
                        y: slot.y + 2,
                    });
                    matched_slot = true;
                    matched_y_outer = true;
                }
            }

            if slot.y > 0
                && in_bounds(slot.x, slot.y - 1)
                && self.gems[slot.x][slot.y - 1].gem_type == gem_type
            {
                if slot.y > 1
                    && in_bounds(slot.x, slot.y - 2)
                    && self.gems[slot.x][slot.y - 2].gem_type == gem_type
                {
                    matched.push(GridPosition {
                        x: slot.x,
                        y: slot.y - 1,
                    });
                    matched.push(GridPosition {
                        x: slot.x,
                        y: slot.y - 2,
                    });
                    if matched_y_inner && !matched_y_outer {
                        matched.push(GridPosition {
                            x: slot.x,
                            y: slot.y + 1,
                        });
                    }
                    matched_slot = true;
                } else if matched_y_inner {
                    matched_slot = true;
                    matched.push(GridPosition {
                        x: slot.x,
                        y: slot.y - 1,
                    });
                    if !matched_y_outer {
                        matched.push(GridPosition {
                            x: slot.x,
                            y: slot.y + 1,
                        });
                    }
                }
            }

            if matched_slot {
                matched.push(GridPosition {
                    x: slot.x,
                    y: slot.y,
                });
            }
        }

        matched
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
