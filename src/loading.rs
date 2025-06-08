use crate::{gems::GemType, GameState};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    pub background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "textures/head.png")]
    pub head: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 9, rows = 2))]
    pub head_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/head2.png")]
    pub head2: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 9, rows = 2))]
    pub head2_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/tail.png")]
    pub tail: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 9, rows = 2))]
    pub tail_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/tail2.png")]
    pub tail2: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 9, rows = 2))]
    pub tail2_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/body.png")]
    pub body: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 9, rows = 2))]
    pub body_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/tile.png")]
    pub tile: Handle<Image>,
    #[asset(path = "textures/active.png")]
    pub _active: Handle<Image>,
    #[asset(path = "textures/collision.png")]
    pub _collision: Handle<Image>,
    #[asset(path = "textures/gem1.png")]
    pub gem1: Handle<Image>,
    #[asset(path = "textures/gem2.png")]
    pub gem2: Handle<Image>,
    #[asset(path = "textures/gem3.png")]
    pub gem3: Handle<Image>,
    #[asset(path = "textures/gem4.png")]
    pub gem4: Handle<Image>,
    #[asset(path = "textures/gem5.png")]
    pub gem5: Handle<Image>,
}

impl TextureAssets {
    pub fn gem(&self, gem: &GemType) -> Handle<Image> {
        match *gem {
            GemType::One => self.gem1.clone(),
            GemType::Two => self.gem2.clone(),
            GemType::Three => self.gem3.clone(),
            GemType::Four => self.gem4.clone(),
            GemType::Five => self.gem5.clone(),
        }
    }
}
