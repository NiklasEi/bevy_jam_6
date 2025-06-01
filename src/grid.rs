use bevy::prelude::*;

pub struct GridPlugin;

pub const GRID_WIDTH: usize = 10;
pub const GRID_HEIGHT: usize = 10;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {}
}
