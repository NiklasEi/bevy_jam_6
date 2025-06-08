use bevy::prelude::*;

use crate::{player::GrowthTimer, GameState};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnakeLength>()
            .init_resource::<Explosions>()
            .init_resource::<ExplosionsTotal>()
            .init_resource::<BiggestChainReaction>()
            .init_resource::<MaxSnakeLength>()
            .add_systems(OnExit(GameState::Menu), setup)
            .add_systems(
                Update,
                (update_max_length, (update_game_ui, update_other_game_ui))
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct BiggestChainReaction(pub usize);

#[derive(Resource, Default)]
pub struct Explosions(pub usize);

#[derive(Resource, Default)]
pub struct ExplosionsTotal(pub usize);

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
struct BiggestChainReactionText;

#[derive(Component)]
struct ExplosionsText;

#[derive(Component)]
struct ExplosionsTotalText;

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
            left: Val::Px(270.0),
            ..default()
        },
        MaxLengthText,
    ));
    commands.spawn((
        Text::new("Gems destroyed: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(35.0),
            left: Val::Px(12.0),
            ..default()
        },
        ExplosionsText,
    ));
    commands.spawn((
        Text::new("Total destroyed: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(35.0),
            left: Val::Px(270.0),
            ..default()
        },
        ExplosionsTotalText,
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
    commands.spawn((
        Text::new("Longest chain: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(35.0),
            right: Val::Px(12.0),
            ..default()
        },
        BiggestChainReactionText,
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

fn update_other_game_ui(
    mut explosions_text: Query<
        &mut Text,
        (
            With<ExplosionsText>,
            Without<ExplosionsTotalText>,
            Without<BiggestChainReactionText>,
        ),
    >,
    mut explosions_total_text: Query<
        &mut Text,
        (With<ExplosionsTotalText>, Without<BiggestChainReactionText>),
    >,
    mut biggest_chain_reaction_text: Query<&mut Text, With<BiggestChainReactionText>>,
    explosions: Res<Explosions>,
    explosions_total: Res<ExplosionsTotal>,
    biggest_chain_reaction: Res<BiggestChainReaction>,
) -> Result {
    **explosions_text.single_mut()? = format!("Gems destroyed: {}", explosions.0);
    **explosions_total_text.single_mut()? = format!("Total destroyed: {}", explosions_total.0);
    **biggest_chain_reaction_text.single_mut()? =
        format!("Longest chain: {}", biggest_chain_reaction.0);

    Ok(())
}
