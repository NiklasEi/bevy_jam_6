use std::time::Duration;

use crate::actions::{MoveDirection, NextMove, Player};
use crate::following::Trailing;
use crate::grid::random_placement;
use crate::loading::TextureAssets;
use crate::movement::MovementTimer;
use crate::GameState;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Actions;
use bevy_rand::global::GlobalEntropy;
use bevy_rand::prelude::ChaCha8Rng;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (move_player, update_player_direction).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct SnakeHead;

#[derive(Component)]
pub struct SnakeTail;

fn spawn_player(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut rng: GlobalEntropy<ChaCha8Rng>,
) {
    let mut placements = random_placement(3, &mut rng);
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
    let body = commands
        .spawn((
            Sprite::from_atlas_image(
                textures.body.clone(),
                TextureAtlas {
                    index: 0,
                    layout: textures.body_layout.clone(),
                },
            ),
            placement.2,
            NextMove(placement.1),
            MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
            placement.0,
            Trailing(head),
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
        Trailing(body),
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

fn move_player(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &NextMove), With<Actions<Player>>>,
) {
    for (mut player_transform, next_move) in &mut player_query {
        // warn!("next move: {next_move:?}");
    }
}
