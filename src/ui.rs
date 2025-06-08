use bevy::prelude::*;

use crate::{player::GrowthTimer, GameState};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnakeLength>()
            .init_resource::<MaxSnakeLength>()
            .add_systems(OnExit(GameState::Menu), setup)
            .add_systems(
                Update,
                (update_max_length, update_game_ui)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct SnakeLength(pub usize);

#[derive(Resource, Default)]
pub struct MaxSnakeLength(pub usize);

fn update_max_length(length: Res<SnakeLength>, mut max_length: ResMut<MaxSnakeLength>) {
    if length.0 > max_length.0 {
        max_length.0 = length.0;
    }
}

#[derive(Component)]
struct CurrentLengthText;

#[derive(Component)]
struct MaxLengthText;

#[derive(Component)]
struct NextGrowthText;

fn setup(mut commands: Commands, timer: Res<GrowthTimer>) {
    commands.spawn((
        Text::new("Snake length: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        CurrentLengthText,
    ));
    commands.spawn((
        Text::new("Record length: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(300.0),
            ..default()
        },
        MaxLengthText,
    ));
    commands.spawn((
        Text::new(format!("Next growth: {}s", timer.0.remaining_secs().ceil())),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        NextGrowthText,
    ));
}

fn update_game_ui(
    mut length_text: Query<
        &mut Text,
        (
            With<CurrentLengthText>,
            Without<MaxLengthText>,
            Without<NextGrowthText>,
        ),
    >,
    mut max_length_text: Query<&mut Text, (With<MaxLengthText>, Without<NextGrowthText>)>,
    mut next_growth_text: Query<&mut Text, With<NextGrowthText>>,
    length: Res<SnakeLength>,
    max_length: Res<MaxSnakeLength>,
    timer: Res<GrowthTimer>,
) -> Result {
    **length_text.single_mut()? = format!("Snake length: {}", length.0);
    **max_length_text.single_mut()? = format!("Record length: {}", max_length.0);
    **next_growth_text.single_mut()? = format!("Next growth: {}s", timer.0.remaining_secs().ceil());

    Ok(())
}
