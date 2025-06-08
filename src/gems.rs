use bevy::prelude::*;
use bevy_rand::{global::GlobalEntropy, prelude::ChaCha8Rng};
use rand::Rng;

use crate::{
    board::{fill_board, Board},
    grid::{position_to_transform, GRID_HEIGHT, GRID_WIDTH, TILE_SIZE},
    loading::TextureAssets,
    player::GridPosition,
    GamePhase, GameState,
};

pub struct GemsPlugin;

impl Plugin for GemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), draw_board.after(fill_board))
            .add_systems(Update, start_waiting.run_if(in_state(GamePhase::Playing)))
            .add_systems(
                Update,
                (fall, stop_waiting)
                    .chain()
                    .run_if(in_state(GamePhase::Waiting)),
            )
            .add_systems(OnEnter(GameState::Restarting), remove_gems);
    }
}

fn start_waiting(falling: Query<&Falling>, mut next_state: ResMut<NextState<GamePhase>>) {
    if !falling.is_empty() {
        next_state.set(GamePhase::Waiting);
    }
}

fn stop_waiting(falling: Query<&Falling>, mut next_state: ResMut<NextState<GamePhase>>) {
    if falling.is_empty() {
        next_state.set(GamePhase::Playing);
    }
}

fn fall(
    mut commands: Commands,
    time: Res<Time>,
    gems: Query<(Entity, &mut Transform, &GridPosition), With<Falling>>,
) {
    let speed = 500.;
    for (entity, mut transform, position) in gems {
        let diff = position_to_transform(position).extend(0.) - transform.translation;
        if diff.length() < time.delta_secs() * speed {
            transform.translation = position_to_transform(position).extend(0.);
            commands.entity(entity).remove::<Falling>();
        } else {
            let movement = diff.normalize() * time.delta_secs() * speed;
            transform.translation += movement;
        }
    }
}

#[derive(Component)]
pub struct Falling;

fn draw_board(mut commands: Commands, assets: Res<TextureAssets>, mut board: ResMut<Board>) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let gem_type = board.gems[x][y].gem_type.clone();
            let position = GridPosition { x, y };
            let id = commands
                .spawn((
                    Transform::from_translation(
                        position_to_transform(&position).extend(0.)
                            + Vec3::new(
                                0.,
                                TILE_SIZE * (GRID_HEIGHT + 1) as f32 + y as f32 * TILE_SIZE / 2.,
                                0.,
                            ),
                    ),
                    Sprite::from_image(assets.gem(&gem_type)),
                    gem_type,
                    position,
                    Falling,
                ))
                .id();
            board.gems[x][y].entity = Some(id);
        }
    }
}

fn remove_gems(mut commands: Commands, gems: Query<Entity, With<GemType>>) {
    for gem in gems {
        commands.entity(gem).despawn();
    }
}

#[derive(PartialEq, Eq, Component, Clone)]
pub enum GemType {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl GemType {
    pub fn random(rng: &mut GlobalEntropy<ChaCha8Rng>) -> Self {
        match rng.gen_range(0..5) {
            0 => GemType::One,
            1 => GemType::Two,
            2 => GemType::Three,
            3 => GemType::Four,
            4 => GemType::Five,
            _ => unreachable!(),
        }
    }
}
