use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    grid::{GRID_HEIGHT, GRID_WIDTH},
    player::{GridPosition, SnakeHead},
};

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions.
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<Player>()
            .add_observer(player_binding)
            .add_observer(next_move_straight)
            .add_observer(next_move_right)
            .add_observer(next_move_left);
    }
}

fn player_binding(trigger: Trigger<Binding<Player>>, mut players: Query<&mut Actions<Player>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<MoveStraight>()
        .to((KeyCode::KeyW, KeyCode::ArrowUp, GamepadButton::DPadUp))
        .with_modifiers(DeadZone::default())
        .with_conditions(Pulse::new(0.2));
    actions
        .bind::<MoveRight>()
        .to((KeyCode::KeyD, KeyCode::ArrowRight, GamepadButton::DPadRight))
        .with_modifiers(DeadZone::default())
        .with_conditions(Pulse::new(0.2));
    actions
        .bind::<MoveLeft>()
        .to((KeyCode::KeyA, KeyCode::ArrowLeft, GamepadButton::DPadLeft))
        .with_modifiers(DeadZone::default())
        .with_conditions(Pulse::new(0.2));
}

#[derive(InputContext)]
pub struct Player;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct MoveStraight;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct MoveLeft;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct MoveRight;

#[derive(Component, Debug, Clone, Copy)]
pub enum Orientation {
    Up,
    Right,
    Down,
    Left,
}

impl Orientation {
    pub fn next(&mut self, next: &NextMove) {
        match next.0 {
            MoveDirection::Straight => (),
            MoveDirection::Left => match self {
                Orientation::Up => *self = Orientation::Left,
                Orientation::Left => *self = Orientation::Down,
                Orientation::Down => *self = Orientation::Right,
                Orientation::Right => *self = Orientation::Up,
            },
            MoveDirection::Right => match self {
                Orientation::Up => *self = Orientation::Right,
                Orientation::Right => *self = Orientation::Down,
                Orientation::Down => *self = Orientation::Left,
                Orientation::Left => *self = Orientation::Up,
            },
        }
    }

    pub fn direction(&self) -> Vec3 {
        match self {
            Orientation::Up => Vec3::Y,
            Orientation::Right => Vec3::X,
            Orientation::Down => -Vec3::Y,
            Orientation::Left => -Vec3::X,
        }
    }

    pub fn next_position(&self, position: &GridPosition) -> GridPosition {
        let mut next = position.clone();
        match self {
            Orientation::Up => next.y = (next.y + 1) % GRID_HEIGHT,
            Orientation::Right => next.x = (next.x + 1) % GRID_WIDTH,
            Orientation::Down => {
                if next.y == 0 {
                    next.y = GRID_HEIGHT - 1;
                } else {
                    next.y -= 1;
                }
            }
            Orientation::Left => {
                if next.x == 0 {
                    next.x = GRID_WIDTH - 1;
                } else {
                    next.x -= 1;
                }
            }
        }

        next
    }

    pub fn previous_position(&self, position: &GridPosition) -> GridPosition {
        let mut previous = position.clone();
        match self {
            Orientation::Down => previous.y = (previous.y + 1) % GRID_HEIGHT,
            Orientation::Left => previous.x = (previous.x + 1) % GRID_WIDTH,
            Orientation::Up => {
                if previous.y == 0 {
                    previous.y = GRID_HEIGHT - 1;
                } else {
                    previous.y -= 1;
                }
            }
            Orientation::Right => {
                if previous.x == 0 {
                    previous.x = GRID_WIDTH - 1;
                } else {
                    previous.x -= 1;
                }
            }
        }

        previous
    }
}

#[derive(Component, Debug, Clone)]
pub struct NextMove(pub MoveDirection);

impl NextMove {
    pub fn z_angle(&self) -> f32 {
        match self.0 {
            MoveDirection::Straight => 0.,
            MoveDirection::Right => -PI / 2.,
            MoveDirection::Left => PI / 2.,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Straight,
    Right,
}

fn next_move_straight(
    trigger: Trigger<Fired<MoveStraight>>,
    players: Query<&mut NextMove, With<SnakeHead>>,
) {
    if trigger.value {
        for mut next_move in players {
            next_move.0 = MoveDirection::Straight;
        }
    }
}

fn next_move_left(
    trigger: Trigger<Fired<MoveLeft>>,
    players: Query<&mut NextMove, With<SnakeHead>>,
) {
    if trigger.value {
        for mut next_move in players {
            next_move.0 = MoveDirection::Left;
        }
    }
}

fn next_move_right(
    trigger: Trigger<Fired<MoveRight>>,
    players: Query<&mut NextMove, With<SnakeHead>>,
) {
    if trigger.value {
        info!("turning right");
        for mut next_move in players {
            next_move.0 = MoveDirection::Right;
        }
    }
}
