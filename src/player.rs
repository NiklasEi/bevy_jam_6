use std::time::Duration;

use crate::actions::{MoveDirection, NextMove, Orientation, Player};
use crate::board::fill_board;
use crate::following::Trailing;
use crate::grid::{position_to_transform, random_placement, GRID_HEIGHT, GRID_WIDTH};
use crate::loading::TextureAssets;
use crate::movement::MovementTimer;
use crate::{AppSystems, GamePhase, GameState};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Actions;
use bevy_rand::global::GlobalEntropy;
use bevy_rand::prelude::ChaCha8Rng;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePositions>()
            .init_resource::<SnakePositions>()
            .insert_resource(GrowthTimer(Timer::from_seconds(5., TimerMode::Repeating)))
            .add_systems(OnEnter(GameState::Playing), spawn_player.before(fill_board))
            .add_systems(
                Update,
                (
                    update_player_direction.in_set(AppSystems::Input),
                    (check_collisions, (mark_taken, update_active))
                        .run_if(in_state(GamePhase::Playing))
                        .run_if(resource_changed::<SnakePositions>)
                        .chain()
                        .in_set(AppSystems::CheckCollision),
                    grow_snake.in_set(AppSystems::Spawn),
                )
                    .run_if(in_state(GamePhase::Playing)),
            )
            .add_observer(on_grid_position_insert)
            .add_observer(on_grid_position_replaced);
    }
}

#[derive(Component)]
pub struct SnakeHead;

#[derive(Component)]
pub struct SnakeHeadInner;

#[derive(Component)]
pub struct SnakeTail;

#[derive(Component)]
pub struct SnakeTailInner;

fn spawn_player(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut rng: GlobalEntropy<ChaCha8Rng>,
) {
    commands.insert_resource(SnakePositions::default());
    let mut placements = random_placement(4, &mut rng);
    info!("Starting positions: {placements:?}");
    let mut placement = placements.pop().unwrap();
    let head = commands
        .spawn((
            Sprite::from_atlas_image(
                textures.head.clone(),
                TextureAtlas {
                    index: 0,
                    layout: textures.head_layout.clone(),
                },
            ),
            placement.2,
            placement.3,
            NextMove(placement.1),
            Actions::<Player>::default(),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            SnakeHead,
            placement.0,
            SnakePart,
        ))
        .id();
    placement = placements.pop().unwrap();
    let head2 = commands
        .spawn((
            Sprite::from_atlas_image(
                textures.head2.clone(),
                TextureAtlas {
                    index: 0,
                    layout: textures.head2_layout.clone(),
                },
            ),
            placement.2,
            placement.3,
            NextMove(placement.1),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            placement.0,
            SnakeHeadInner,
            Trailing(head),
            SnakePart,
        ))
        .id();
    placement = placements.pop().unwrap();
    let tail2 = commands
        .spawn((
            Sprite::from_atlas_image(
                textures.tail2.clone(),
                TextureAtlas {
                    index: 0,
                    layout: textures.tail2_layout.clone(),
                },
            ),
            placement.2,
            placement.3,
            NextMove(placement.1),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            placement.0,
            SnakeTailInner,
            Trailing(head2),
            SnakePart,
        ))
        .id();
    placement = placements.pop().unwrap();
    commands.spawn((
        Sprite::from_atlas_image(
            textures.tail.clone(),
            TextureAtlas {
                index: 0,
                layout: textures.tail_layout.clone(),
            },
        ),
        placement.2,
        placement.3,
        NextMove(placement.1),
        MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        placement.0,
        Trailing(tail2),
        SnakeTail,
        SnakePart,
    ));
}

fn update_player_direction(player: Query<(&NextMove, &mut Sprite), Changed<NextMove>>) {
    for (next_move, mut sprite) in player {
        let atlas = sprite.texture_atlas.as_mut().unwrap();
        match next_move.0 {
            MoveDirection::Straight => atlas.index %= 9,
            MoveDirection::Left => {
                atlas.index = 9 + atlas.index % 9;
                sprite.flip_x = false;
            }
            MoveDirection::Right => {
                atlas.index = 9 + atlas.index % 9;
                sprite.flip_x = true;
            }
        }
    }
}

#[derive(Resource)]
struct GrowthTimer(Timer);

/// Marker to not move in the next round to make space for a new snake part
#[derive(Component)]
pub struct StuckOnce;

fn grow_snake(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    inner_tail: Query<
        (
            Entity,
            &Transform,
            &Orientation,
            &NextMove,
            &Sprite,
            &MovementTimer,
            &Trailing,
            &GridPosition,
        ),
        With<SnakeTailInner>,
    >,
    tail: Query<Entity, (With<SnakeTail>, Without<SnakeTailInner>)>,
    time: Res<Time>,
    mut timer: ResMut<GrowthTimer>,
) -> Result {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        let (
            inner_tail,
            transform,
            orientation,
            next_move,
            sprite,
            movement_timer,
            trailing,
            position,
        ) = inner_tail.single()?;
        let new_body_part = commands
            .spawn((
                Sprite::from_atlas_image(
                    textures.body.clone(),
                    TextureAtlas {
                        index: sprite.texture_atlas.as_ref().unwrap().index,
                        layout: textures.body_layout.clone(),
                    },
                ),
                *orientation,
                next_move.clone(),
                {
                    let current_time = movement_timer.0.elapsed();
                    let mut timer =
                        MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating));
                    timer.0.tick(current_time);

                    timer
                },
                *transform,
                position.clone(),
                Trailing(trailing.0),
                Visibility::Hidden,
                NewBody,
                SnakePart,
            ))
            .id();
        commands
            .entity(inner_tail)
            .insert((Trailing(new_body_part), StuckOnce));
        commands.entity(tail.single()?).insert(StuckOnce);
    }

    Ok(())
}

#[derive(Resource, Default, Debug)]
struct SnakePositions([[Vec<Entity>; GRID_HEIGHT]; GRID_WIDTH]);

#[derive(Component, Clone, Debug)]
#[component(immutable)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
struct NewBody;

#[derive(Component)]
pub struct SnakePart;

fn on_grid_position_insert(
    trigger: Trigger<OnInsert, GridPosition>,
    query: Query<&GridPosition, (Without<NewBody>, With<SnakePart>)>,
    mut positions: ResMut<SnakePositions>,
) {
    if let Ok(grid_position) = query.get(trigger.target()) {
        positions.0[grid_position.x][grid_position.y].push(trigger.target());
    }
}

fn on_grid_position_replaced(
    trigger: Trigger<OnReplace, GridPosition>,
    query: Query<&GridPosition, (Without<NewBody>, With<SnakePart>)>,
    new_parts: Query<Entity, With<NewBody>>,
    mut positions: ResMut<SnakePositions>,
    mut commands: Commands,
) {
    if let Ok(grid_position) = query.get(trigger.target()) {
        if let Some(index) = positions.0[grid_position.x][grid_position.y]
            .iter()
            .position(|value| *value == trigger.target())
        {
            positions.0[grid_position.x][grid_position.y].swap_remove(index);
        }
    }

    for new_part in new_parts {
        commands.entity(new_part).remove::<NewBody>();
    }
}

fn check_collisions(positions: Res<SnakePositions>) {
    for (x, column) in positions.0.iter().enumerate() {
        for (y, entities) in column.iter().enumerate() {
            if entities.len() > 1 {
                info!("Collision at {x}/{y}")
            }
        }
    }
}

#[derive(Default, Resource)]
pub struct ActivePositions(pub Vec<GridPosition>);

fn update_active(
    mut active: ResMut<ActivePositions>,
    snake: Query<&GridPosition, With<SnakePart>>,
) {
    active.0.clear();
    for pos in snake {
        active.0.push(pos.clone());
    }
}

#[derive(Component)]
struct ActiveMarker;

fn mark_taken(
    active: Query<Entity, With<ActiveMarker>>,
    mut commands: Commands,
    asset: Res<TextureAssets>,
    positions: Res<SnakePositions>,
) {
    active
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            if positions.0[x][y].len() == 1 {
                commands.spawn((
                    ActiveMarker,
                    Transform::from_translation(
                        position_to_transform(&GridPosition { x, y }).extend(0.),
                    ),
                    Sprite::from_image(asset.active.clone()),
                ));
            } else if positions.0[x][y].len() > 1 {
                commands.spawn((
                    ActiveMarker,
                    Transform::from_translation(
                        position_to_transform(&GridPosition { x, y }).extend(0.),
                    ),
                    Sprite::from_image(asset.collision.clone()),
                ));
            }
        }
    }
}
