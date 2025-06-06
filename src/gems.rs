use bevy::prelude::*;
use bevy_rand::{global::GlobalEntropy, prelude::ChaCha8Rng};
use rand::Rng;

use crate::{
    grid::{position_to_transform, GRID_HEIGHT, GRID_WIDTH},
    loading::TextureAssets,
    player::GridPosition,
    GameState,
};

pub struct GemsPlugin;

impl Plugin for GemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), fill);
    }
}

fn fill(mut commands: Commands, assets: Res<TextureAssets>, mut rng: GlobalEntropy<ChaCha8Rng>) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let gem = Gem::random(&mut rng);
            commands.spawn((
                Transform::from_translation(
                    position_to_transform(&GridPosition { x, y }).extend(0.),
                ),
                Sprite::from_image(assets.gem(&gem)),
                gem,
            ));
        }
    }
}

#[derive(PartialEq, Eq, Component)]
pub enum Gem {
    One,
    Two,
    Three,
    Four,
}

impl Gem {
    fn random(rng: &mut GlobalEntropy<ChaCha8Rng>) -> Self {
        match rng.gen_range(0..4) {
            0 => Gem::One,
            1 => Gem::Two,
            2 => Gem::Three,
            3 => Gem::Four,
            _ => unreachable!(),
        }
    }
}
