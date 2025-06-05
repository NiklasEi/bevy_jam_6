use std::time::Duration;

use crate::actions::{MoveDirection, NextMove, Orientation, Player};
use crate::following::Trailing;
use crate::grid::random_placement;
use crate::loading::TextureAssets;
use crate::movement::MovementTimer;
use crate::{AppSystems, GameState};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Actions;
use bevy_rand::global::GlobalEntropy;
use bevy_rand::prelude::ChaCha8Rng;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GrowthTimer(Timer::from_seconds(5., TimerMode::Repeating)))
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    update_player_direction.in_set(AppSystems::Input),
                    grow_snake.in_set(AppSystems::Move),
                )
                    .run_if(in_state(GameState::Playing)),
            );
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
            NextMove(placement.1),
            Actions::<Player>::default(),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            SnakeHead,
            placement.0,
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
            NextMove(placement.1),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            placement.0,
            SnakeHeadInner,
            Trailing(head),
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
            NextMove(placement.1),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            placement.0,
            SnakeTailInner,
            Trailing(head2),
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
        NextMove(placement.1),
        MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        placement.0,
        Trailing(tail2),
        SnakeTail,
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
        ),
        With<SnakeTailInner>,
    >,
    tail: Query<Entity, (With<SnakeTail>, Without<SnakeTailInner>)>,
    time: Res<Time>,
    mut timer: ResMut<GrowthTimer>,
) -> Result {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        let (inner_tail, transform, orientation, next_move, sprite, movement_timer, trailing) =
            inner_tail.single()?;
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
                Trailing(trailing.0),
                Visibility::Hidden,
            ))
            .id();
        commands
            .entity(inner_tail)
            .insert((Trailing(new_body_part), StuckOnce));
        commands.entity(tail.single()?).insert(StuckOnce);
    }

    Ok(())
}
