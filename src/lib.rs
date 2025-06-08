#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod board;
mod following;
mod gems;
mod grid;
mod loading;
mod menu;
mod movement;
mod player;

use crate::actions::ActionsPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::movement::MovementPlugin;
use crate::player::PlayerPlugin;

use audio::InternalAudioPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputSystem;
use board::BoardPlugin;
use gems::GemsPlugin;
use grid::GridPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    Playing,
    Menu,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
enum GamePhase {
    #[default]
    Playing,
    Exploding,
    Waiting,
    Pause,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<GamePhase>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                PlayerPlugin,
                MovementPlugin,
                GridPlugin,
                InternalAudioPlugin,
                GemsPlugin,
                BoardPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
            ));
        }
        app.configure_sets(
            Update,
            (
                EnhancedInputSystem,
                AppSystems::Input,
                AppSystems::Move,
                AppSystems::Spawn,
                AppSystems::CheckCollision,
                AppSystems::Match,
            )
                .chain(),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    Input,
    Move,
    Spawn,
    CheckCollision,
    Match,
}
