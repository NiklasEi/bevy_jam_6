use bevy::prelude::*;

use crate::{loading::AudioAssets, GameState};

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), start_audio);
    }
}

fn start_audio(audio: Res<AudioAssets>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(audio.background.clone()),
        PlaybackSettings::LOOP,
    ));
}
