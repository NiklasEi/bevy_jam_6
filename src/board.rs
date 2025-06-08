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
                (
                    explode
                        .in_set(AppSystems::Match)
                        .run_if(in_state(GamePhase::Playing))
                        .run_if(resource_changed::<ActivePositions>),
                    animate_exploding_gems
                        .in_set(AppSystems::Match)
                        .run_if(in_state(GamePhase::Exploding)),
                ),
            )
            .add_systems(OnEnter(GamePhase::Exploding), reset_exploding_timer);
    }
}

#[derive(Component)]
struct Exploding(pub u8);

fn explode(
    head: Query<&GridPosition, With<SnakeHead>>,
    mut board: ResMut<Board>,
    mut commands: Commands,
    // exploding: Query<Entity, With<Exploding>>,
    asset: Res<TextureAssets>,
    mut rng: GlobalEntropy<ChaCha8Rng>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) -> Result {
    // exploding
    //     .iter()
    //     .for_each(|entity| commands.entity(entity).despawn());

    let mut checked = [[false; GRID_HEIGHT]; GRID_WIDTH];
    let mut exploding = [[0; GRID_HEIGHT]; GRID_WIDTH];
    let mut active = vec![head.single()?.clone()];
    let mut iteration = 0u8;
    let mut found = false;
    loop {
        iteration += 1;
        let new_possitions = board.find_matches(iteration, &active, &mut checked, &mut exploding);
        if new_possitions.is_empty() {
            break;
        }
        found = true;
        let neighbors = GridPosition::surroundings(&new_possitions.into_iter().collect::<Vec<_>>());
        active = neighbors.into_iter().collect::<Vec<_>>();
    }
    info!("Did {iteration} iterations!");

    if !found {
        return Ok(());
    }
    next_phase.set(GamePhase::Exploding);

    #[allow(clippy::needless_range_loop)]
    for column in 0..GRID_WIDTH {
        let mut spawn_count = 0;
        for y in 0..GRID_HEIGHT {
            let Some(entity) = board.gems[column][y].entity else {
                error!("Missing gem entity");
                continue;
            };
            if exploding[column][y] > 0 {
                spawn_count += 1;
                commands
                    .entity(entity)
                    .insert(Exploding(exploding[column][y]));
                // commands.spawn((
                //     Exploding,
                //     Transform::from_translation(
                //         position_to_transform(&GridPosition { x: column, y }).extend(0.),
                //     ),
                //     Sprite::from_image(asset.collision.clone()),
                // ));
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
                                    + (spawn_count - spawn) as f32 * TILE_SIZE / 2.,
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

#[derive(Resource)]
struct ExplodingTimer(Timer);

fn reset_exploding_timer(mut commands: Commands) {
    commands.insert_resource(ExplodingTimer(Timer::from_seconds(
        0.3,
        TimerMode::Repeating,
    )));
}

fn animate_exploding_gems(
    exploding: Query<(Entity, &mut Exploding)>,
    mut commands: Commands,
    mut timer: ResMut<ExplodingTimer>,
    time: Res<Time>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    if exploding.is_empty() {
        next_phase.set(GamePhase::Waiting);
    }
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        for (entity, mut exploding) in exploding {
            if exploding.0 == 1 {
                commands.entity(entity).despawn();
            } else {
                exploding.0 -= 1;
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

    fn find_matches(
        &self,
        iteration: u8,
        active_slots: &Vec<GridPosition>,
        checked: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
        exploding: &mut [[u8; GRID_HEIGHT]; GRID_WIDTH],
    ) -> HashSet<GridPosition> {
        let mut positions = HashSet::default();
        for slot in active_slots {
            self.check_slot(iteration, slot, checked, exploding, &mut positions);
        }

        positions
    }

    fn check_slot(
        &self,
        iteration: u8,
        slot: &GridPosition,
        checked: &mut [[bool; GRID_HEIGHT]; GRID_WIDTH],
        exploding: &mut [[u8; GRID_HEIGHT]; GRID_WIDTH],
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
                    mark_for_explosion(slot.x + 1, slot.y, iteration, positions, exploding);
                    mark_for_explosion(slot.x + 2, slot.y, iteration, positions, exploding);
                }
                i => mark_for_explosion(slot.x + i, slot.y, iteration, positions, exploding),
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
                        mark_for_explosion(slot.x - 1, slot.y, iteration, positions, exploding);
                        if !matched_x_slot {
                            matched_slot = true;
                            mark_for_explosion(slot.x + 1, slot.y, iteration, positions, exploding);
                        }
                    }
                }
                2 => {
                    matched_slot = true;
                    if !matched_x_plus {
                        mark_for_explosion(slot.x - 1, slot.y, iteration, positions, exploding);
                    }
                    mark_for_explosion(slot.x - 2, slot.y, iteration, positions, exploding);
                }
                i => mark_for_explosion(slot.x - i, slot.y, iteration, positions, exploding),
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
                    mark_for_explosion(slot.x, slot.y + 1, iteration, positions, exploding);
                    mark_for_explosion(slot.x, slot.y + 2, iteration, positions, exploding);
                }
                i => mark_for_explosion(slot.x, slot.y + i, iteration, positions, exploding),
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
                        mark_for_explosion(slot.x, slot.y - 1, iteration, positions, exploding);
                        if !matched_y_slot {
                            matched_slot = true;
                            mark_for_explosion(slot.x, slot.y + 1, iteration, positions, exploding);
                        }
                    }
                }
                2 => {
                    matched_slot = true;
                    if !matched_y_plus {
                        mark_for_explosion(slot.x, slot.y - 1, iteration, positions, exploding);
                    }
                    mark_for_explosion(slot.x, slot.y - 2, iteration, positions, exploding);
                }
                i => mark_for_explosion(slot.x, slot.y - i, iteration, positions, exploding),
            };
            y_diff += 1;
        }

        if matched_slot {
            mark_for_explosion(slot.x, slot.y, iteration, positions, exploding);
        }

        matched_slot
    }
}

fn mark_for_explosion(
    x: usize,
    y: usize,
    iteration: u8,
    positions: &mut HashSet<GridPosition>,
    exploding: &mut [[u8; GRID_HEIGHT]; GRID_WIDTH],
) {
    if exploding[x][y] == 0 {
        exploding[x][y] = iteration;
    }
    positions.insert(GridPosition { x, y });
}

fn in_bounds(x: usize, y: usize) -> bool {
    x < GRID_WIDTH && y < GRID_HEIGHT
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
