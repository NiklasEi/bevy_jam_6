use std::time::Duration;

use crate::actions::{MoveDirection, NextMove, Orientation, Player};
use crate::loading::TextureAssets;
use crate::movement::MovementTimer;
use crate::GameState;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Actions;

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

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        Sprite::from_atlas_image(
            textures.head.clone(),
            TextureAtlas {
                index: 0,
                layout: textures.head_layout.clone(),
            },
        ),
        Transform::from_translation(Vec3::new(0., 0., 1.)),
        NextMove(MoveDirection::Straight),
        Actions::<Player>::default(),
        MovementTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        SnakeHead,
        Orientation::Up,
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
