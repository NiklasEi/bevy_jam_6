use bevy::prelude::*;

use crate::{loading::AudioAssets, GameState};

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffect>()
            .add_systems(OnExit(GameState::Loading), start_audio)
            .add_systems(
                Update,
                play_sound_effect.run_if(resource_exists::<AudioAssets>),
            );
    }
}

fn play_sound_effect(
    mut events: EventReader<SoundEffect>,
    audio_assets: Res<AudioAssets>,
    mut commands: Commands,
) {
    for event in events.read() {
        let sound = match event {
            SoundEffect::Lost => audio_assets.lost.clone(),
            SoundEffect::Click => audio_assets.click.clone(),
            SoundEffect::Grow => audio_assets.grow.clone(),
            SoundEffect::GemMatch => audio_assets.gem_match.clone(),
            SoundEffect::NomNom => audio_assets.nomnom.clone(),
        };
        commands.spawn(AudioPlayer::new(sound));
    }
}

fn start_audio(audio: Res<AudioAssets>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(audio.background.clone()),
        PlaybackSettings::LOOP,
    ));
}

#[derive(Event)]
pub enum SoundEffect {
    Lost,
    Click,
    Grow,
    GemMatch,
    NomNom,
}
