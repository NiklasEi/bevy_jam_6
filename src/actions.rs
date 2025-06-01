use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

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
        .with_modifiers(DeadZone::default());
    actions
        .bind::<MoveRight>()
        .to((KeyCode::KeyD, KeyCode::ArrowRight, GamepadButton::DPadRight))
        .with_modifiers(DeadZone::default());
    actions
        .bind::<MoveLeft>()
        .to((KeyCode::KeyA, KeyCode::ArrowLeft, GamepadButton::DPadLeft))
        .with_modifiers(DeadZone::default());
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

#[derive(Component, Debug)]
pub struct NextMove(pub MoveDirection);

#[derive(Debug)]
pub enum MoveDirection {
    Left,
    Straight,
    Right,
}

fn next_move_straight(trigger: Trigger<Fired<MoveStraight>>, players: Query<&mut NextMove>) {
    if trigger.value {
        for mut next_move in players {
            next_move.0 = MoveDirection::Straight;
        }
    }
}

fn next_move_left(trigger: Trigger<Fired<MoveLeft>>, players: Query<&mut NextMove>) {
    if trigger.value {
        for mut next_move in players {
            next_move.0 = MoveDirection::Left;
        }
    }
}

fn next_move_right(trigger: Trigger<Fired<MoveRight>>, players: Query<&mut NextMove>) {
    if trigger.value {
        for mut next_move in players {
            next_move.0 = MoveDirection::Right;
        }
    }
}
