use crate::{
    gems::{Falling, GemType},
    grid::{position_to_transform, GRID_HEIGHT, GRID_WIDTH, TILE_SIZE},
    loading::TextureAssets,
    player::{ActivePositions, GridPosition, SnakeHead},
    AppSystems, GamePhase, GameState,
};
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_rand::{global::GlobalEntropy, prelude::ChaCha8Rng};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), fill_board)
            .add_systems(
                Update,
                explode
                    .in_set(AppSystems::Match)
                    .run_if(in_state(GamePhase::Playing))
                    .run_if(resource_changed::<ActivePositions>),
            );
    }
}

#[derive(Component)]
struct Exploding;

fn explode(
    head: Query<&GridPosition, With<SnakeHead>>,
    mut board: ResMut<Board>,
    mut commands: Commands,
    exploding: Query<Entity, With<Exploding>>,
    asset: Res<TextureAssets>,
    mut rng: GlobalEntropy<ChaCha8Rng>,
) -> Result {
    exploding
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());

    let mut checked = [[false; GRID_HEIGHT]; GRID_WIDTH];
    let mut exploding = [[false; GRID_HEIGHT]; GRID_WIDTH];
    let mut active = vec![head.single()?.clone()];
    let mut iteration = 0;
    loop {
        let new_possitions = board.find_matches(&active, &mut checked, &mut exploding);
        iteration += 1;
        if new_possitions.is_empty() {
            break;
        }
        let neighbors = GridPosition::surroundings(&new_possitions.into_iter().collect::<Vec<_>>());
        active = neighbors.into_iter().collect::<Vec<_>>();
    }
    info!("Did {iteration} iterations!");

    for column in 0..GRID_WIDTH {
        let mut spawn_count = 0;
        for y in 0..GRID_HEIGHT {
            let Some(entity) = board.gems[column][y].entity else {
                error!("Missing gem entity");
                continue;
            };
            if exploding[column][y] {
                spawn_count += 1;
                commands.entity(entity).despawn();
                commands.spawn((
                    Exploding,
                    Transform::from_translation(
                        position_to_transform(&GridPosition { x: column, y }).extend(0.),
                    ),
                    Sprite::from_image(asset.collision.clone()),
                ));
            } else if spawn_count > 0 {
                commands.entity(entity).insert((
                    Falling,
                    GridPosition {
                        x: column,
                        y: y - spawn_count,
                    },
                ));
                board.gems[column][y - spawn_count] = board.gems[column][y].clone();
            }
        }
        for spawn in 1..=spawn_count {
            let gem_type = GemType::random(&mut rng);
            let position = GridPosition {
                x: column,
                y: GRID_HEIGHT - spawn,
            };
            let id = commands
                .spawn((
                    Transform::from_translation(
                        position_to_transform(&position).extend(0.)
                            + Vec3::new(
                                0.,
                                TILE_SIZE * (GRID_HEIGHT as f32) / 2.
                                    + spawn as f32 * TILE_SIZE / 2.,
                                0.,
                            ),
                    ),
                    Sprite::from_image(asset.gem(&gem_type)),
                    gem_type.clone(),
                    position,
                    Falling,
                ))
                .id();
            board.gems[column][GRID_HEIGHT - spawn] = Gem {
                gem_type,
                entity: Some(id),
            };
        }
    }

    Ok(())
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

    fn find_matches(
        &self,
        active_slots: &Vec<GridPosition>,
        checked: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
        exploding: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
    ) -> HashSet<GridPosition> {
        let mut positions = HashSet::default();
        for slot in active_slots {
            self.check_slot(&slot, checked, exploding, &mut positions);
        }

        positions
    }

    fn check_slot(
        &self,
        slot: &GridPosition,
        checked: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
        exploding: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
        positions: &mut HashSet<GridPosition>,
    ) -> bool {
        if checked[slot.x][slot.y] {
            return false;
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
                    mark_for_explosion(slot.x + 1, slot.y, positions, exploding);
                    mark_for_explosion(slot.x + 2, slot.y, positions, exploding);
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
                        mark_for_explosion(slot.x - 1, slot.y, positions, exploding);
                        if !matched_x_slot {
                            matched_slot = true;
                            mark_for_explosion(slot.x + 1, slot.y, positions, exploding);
                        }
                    }
                }
                2 => {
                    matched_slot = true;
                    if !matched_x_plus {
                        mark_for_explosion(slot.x - 1, slot.y, positions, exploding);
                    }
                    mark_for_explosion(slot.x - 2, slot.y, positions, exploding);
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
                    mark_for_explosion(slot.x, slot.y + 1, positions, exploding);
                    mark_for_explosion(slot.x, slot.y + 2, positions, exploding);
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
                        mark_for_explosion(slot.x, slot.y - 1, positions, exploding);
                        if !matched_y_slot {
                            matched_slot = true;
                            mark_for_explosion(slot.x, slot.y + 1, positions, exploding);
                        }
                    }
                }
                2 => {
                    matched_slot = true;
                    if !matched_y_plus {
                        mark_for_explosion(slot.x, slot.y - 1, positions, exploding);
                    }
                    mark_for_explosion(slot.x, slot.y - 2, positions, exploding);
                }
                i => exploding[slot.x][slot.y - i] = true,
            };
            y_diff += 1;
        }

        if matched_slot {
            mark_for_explosion(slot.x, slot.y, positions, exploding);
        }

        matched_slot
    }
}

fn mark_for_explosion(
    x: usize,
    y: usize,
    positions: &mut HashSet<GridPosition>,
    exploding: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
) {
    exploding[x][y] = true;
    positions.insert(GridPosition { x, y });
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
