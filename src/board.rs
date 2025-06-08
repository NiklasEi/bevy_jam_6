use std::ops::Deref;

use crate::{
    actions::Orientation,
    audio::SoundEffect,
    gems::{Falling, GemType},
    grid::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE},
    loading::TextureAssets,
    player::{ActivePositions, GridPosition, SnakeHead, SnakePart, SnakeTail},
    ui::{BiggestChainReaction, Explosions, ExplosionsTotal},
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
                    tail_manipulation
                        .in_set(AppSystems::Manipulate)
                        .run_if(in_state(GamePhase::Playing))
                        .run_if(|exploding: Query<&Exploding>| exploding.is_empty()),
                    animate_exploding_gems
                        .in_set(AppSystems::Match)
                        .run_if(in_state(GamePhase::Exploding)),
                ),
            )
            .add_systems(OnEnter(GamePhase::Exploding), reset_exploding_timer);
    }
}

fn tail_manipulation(
    mut last_checked: Local<GridPosition>,
    mut board: ResMut<Board>,
    tail: Query<(&GridPosition, &Orientation), (With<SnakeTail>, Changed<GridPosition>)>,
    mut commands: Commands,
) -> Result {
    let Ok((new_position, orientation)) = tail.single() else {
        return Ok(());
    };

    if last_checked.deref() == new_position {
        return Ok(());
    }

    *last_checked = new_position.clone();
    info!("Checking for switch");
    let position = orientation.previous_position(new_position);

    let matches = board.find_matches(
        1,
        &vec![position.clone()],
        &mut [[false; GRID_HEIGHT]; GRID_WIDTH],
        &mut [[0; GRID_HEIGHT]; GRID_WIDTH],
    );
    if !matches.is_empty() {
        return Ok(());
    }

    let neighboors = GridPosition::surroundings(&vec![position.clone()]);
    for target in neighboors {
        if &target == new_position {
            continue;
        }
        let matches = board.find_matches(
            1,
            &vec![target.clone()],
            &mut [[false; GRID_HEIGHT]; GRID_WIDTH],
            &mut [[0; GRID_HEIGHT]; GRID_WIDTH],
        );
        if !matches.is_empty() {
            continue;
        }

        let mut new_board = board.clone();
        let center = new_board.gems[position.x][position.y].clone();
        new_board.gems[position.x][position.y] = new_board.gems[target.x][target.y].clone();
        new_board.gems[target.x][target.y] = center;
        let matches = new_board.find_matches(
            1,
            &vec![position.clone(), target.clone()],
            &mut [[false; GRID_HEIGHT]; GRID_WIDTH],
            &mut [[0; GRID_HEIGHT]; GRID_WIDTH],
        );
        if !matches.is_empty() {
            info!(
                "Switching {}/{} with {}/{} due to match",
                position.x, position.y, target.x, target.y
            );
            commands
                .entity(new_board.gems[target.x][target.y].entity.unwrap())
                .insert((
                    GridPosition {
                        x: target.x,
                        y: target.y,
                    },
                    Falling,
                ));
            commands
                .entity(new_board.gems[position.x][position.y].entity.unwrap())
                .insert((
                    GridPosition {
                        x: position.x,
                        y: position.y,
                    },
                    Falling,
                ));
            *board = new_board;
            return Ok(());
        }
    }

    Ok(())
}

#[derive(Component)]
struct Exploding(pub u8);

#[allow(clippy::too_many_arguments)]
fn explode(
    head: Query<&GridPosition, With<SnakeHead>>,
    mut board: ResMut<Board>,
    mut commands: Commands,
    asset: Res<TextureAssets>,
    mut rng: GlobalEntropy<ChaCha8Rng>,
    mut next_phase: ResMut<NextState<GamePhase>>,
    mut explosions: ResMut<Explosions>,
    mut explosions_total: ResMut<ExplosionsTotal>,
    mut biggest_chain_reaction: ResMut<BiggestChainReaction>,
) -> Result {
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

    let mut count = 0;
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
        count += spawn_count;
        for spawn in 1..=spawn_count {
            let gem_type = GemType::random(&mut rng);
            let position = GridPosition {
                x: column,
                y: GRID_HEIGHT - spawn,
            };
            let id = commands
                .spawn((
                    Transform::from_xyz(
                        (-(GRID_WIDTH as f32) / 2. + position.x as f32 + 0.5) * TILE_SIZE,
                        TILE_SIZE * (GRID_HEIGHT as f32) / 2.
                            + (spawn_count - spawn + 3) as f32 * TILE_SIZE / 2.,
                        0.,
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

    explosions.0 += count;
    explosions_total.0 += count;
    if count > biggest_chain_reaction.0 {
        biggest_chain_reaction.0 = count;
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
    exploding: Query<(Entity, &GridPosition, &mut Exploding), Without<SnakePart>>,
    snake_body: Query<&GridPosition, (With<SnakePart>, Without<SnakeHead>)>,
    mut commands: Commands,
    mut timer: ResMut<ExplodingTimer>,
    time: Res<Time>,
    mut next_phase: ResMut<NextState<GamePhase>>,
    mut writer: EventWriter<SoundEffect>,
) {
    if exploding.is_empty() {
        next_phase.set(GamePhase::Waiting);
    }
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        writer.write(SoundEffect::GemMatch);
        for (entity, position, mut exploding) in exploding {
            if exploding.0 == 1 {
                if snake_body.iter().any(|body| body == position) {
                    info!("Snake got hit by match at {}/{}", position.x, position.y);
                    next_phase.set(GamePhase::Lost);
                    writer.write(SoundEffect::Lost);
                }
                commands.entity(entity).despawn();
            } else {
                exploding.0 -= 1;
            }
        }
    }
}

#[derive(Resource, Default, Clone)]
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

pub fn fill_board(
    mut commands: Commands,
    mut rng: GlobalEntropy<ChaCha8Rng>,
    snake_head: Query<&GridPosition, With<SnakeHead>>,
) -> Result {
    let head = snake_head.single()?;
    let mut board = Board::default();
    board.randomize_gems(&mut rng);
    let surroundings = GridPosition::surroundings(&vec![head.clone()])
        .into_iter()
        .collect::<Vec<_>>();

    let mut rounds = 0;
    loop {
        let matches = board.find_matches(
            1,
            &surroundings,
            &mut [[false; GRID_HEIGHT]; GRID_WIDTH],
            &mut [[0; GRID_HEIGHT]; GRID_WIDTH],
        );
        if matches.is_empty() {
            break;
        }
        rounds += 1;
        board.randomize_gems(&mut rng);
    }

    info!("Took {rounds} rounds to find valid board");
    commands.insert_resource(board);

    Ok(())
}
