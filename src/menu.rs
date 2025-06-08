use crate::audio::SoundEffect;
use crate::loading::TextureAssets;
use crate::{GamePhase, GameState};
use bevy::color::palettes::tailwind::SLATE_200;
use bevy::prelude::*;

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), (camera, setup_menu))
            .add_systems(Update, click_play_button.run_if(in_state(GameState::Menu)))
            .add_systems(Update, click_play_button.run_if(in_state(GamePhase::Lost)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(Update, start_pause.run_if(in_state(GamePhase::Playing)))
            .add_systems(Update, stop_pause.run_if(in_state(GamePhase::Pause)))
            .add_systems(OnEnter(GamePhase::Lost), setup_menu)
            .add_systems(OnExit(GamePhase::Lost), cleanup_menu);
    }
}

fn start_pause(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GamePhase>>) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(GamePhase::Pause);
    }
}

fn stop_pause(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GamePhase>>) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(GamePhase::Playing);
    }
}

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::linear_rgba(0.15, 0.15, 0.15, 0.5),
            hovered: Color::linear_rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component)]
struct Menu;

fn camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
}

fn setup_menu(mut commands: Commands, textures: Res<TextureAssets>, state: Res<State<GameState>>) {
    info!("menu");
    let mut background = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Menu,
    ));
    if state.get() == &GameState::Playing {
        background.insert(BackgroundColor(Color::Srgba(SLATE_200.with_alpha(0.2))));
    }
    background.with_children(|children| {
        let button_colors = ButtonColors::default();
        let mut button = children.spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(button_colors.normal),
            BorderRadius::all(Val::Px(10.)),
            button_colors,
        ));

        if state.get() == &GameState::Playing {
            button.insert(ChangeState(GameState::Restarting));
        } else {
            button.insert(ChangeState(GameState::Playing));
        }
        button.with_child((
            Text::new(if state.get() == &GameState::Menu {
                "Play"
            } else {
                "Try again"
            }),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
        ));
    });
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                bottom: Val::Px(5.),
                width: Val::Percent(100.),
                position_type: PositionType::Absolute,
                ..default()
            },
            ZIndex(1),
            Menu,
        ))
        .with_children(|children| {
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(190.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(5.)),
                        ..Default::default()
                    },
                    BorderRadius::all(Val::Px(10.)),
                    BackgroundColor(Color::NONE),
                    ButtonColors {
                        normal: Color::NONE,
                        ..default()
                    },
                    OpenLink("https://bevyengine.org"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Made with Bevy"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                    parent.spawn((
                        ImageNode {
                            image: textures.bevy.clone(),
                            ..default()
                        },
                        Node {
                            width: Val::Px(32.),
                            ..default()
                        },
                    ));
                });
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(190.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.)),
                    BackgroundColor(Color::NONE),
                    ButtonColors {
                        normal: Color::NONE,
                        hovered: Color::linear_rgb(0.25, 0.25, 0.25),
                    },
                    OpenLink("https://github.com/NiklasEi/bevy_jam_6"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Open source"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                    parent.spawn((
                        ImageNode::new(textures.github.clone()),
                        Node {
                            width: Val::Px(32.),
                            ..default()
                        },
                    ));
                });
        });
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut writer: EventWriter<SoundEffect>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if input.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Restarting);
        writer.write(SoundEffect::Click);
        return;
    }
    for (interaction, mut color, button_colors, change_state, open_link) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                writer.write(SoundEffect::Click);
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}
