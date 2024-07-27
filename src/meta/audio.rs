use bevy::audio::{PlaybackMode, Volume};
use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Deref)]
struct BackgroundAudio(String);

//-------------------------------------------------------------------------------------------------------------------

fn insert_audio(ec: &mut EntityCommands, soundtrack: &Soundtrack, audio: &AudioMap)
{
    ec.try_insert((
        AudioBundle {
            source: audio.get(&soundtrack.source),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(soundtrack.volume),
                ..default()
            },
            ..default()
        },
        BackgroundAudio(soundtrack.source.clone()),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

fn set_soundtrack(
    mut c: Commands,
    soundtracks: Res<SoundtrackDatabase>,
    audio: Res<AudioMap>,
    day: ReactRes<Day>,
    query: Query<(Entity, &BackgroundAudio)>,
)
{
    let Some(current_track) = soundtracks.get(day.get()) else { return };
    let Ok((entity, background)) = query.get_single() else {
        let mut ec = c.spawn_empty();
        insert_audio(&mut ec, current_track, &audio);
        return;
    };
    if **background == current_track.source {
        return;
    }
    let mut ec = c.entity(entity);
    insert_audio(&mut ec, current_track, &audio);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Soundtrack
{
    pub source: String,
    pub volume: f32,
}

//-------------------------------------------------------------------------------------------------------------------

/// Soundtracks loaded in order of which day they should play.
#[derive(Resource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundtrackDatabase
{
    pub tracks: Vec<Soundtrack>,
}

impl SoundtrackDatabase
{
    pub fn get(&self, day: usize) -> Option<&Soundtrack>
    {
        if self.tracks.is_empty() {
            return None;
        }
        let index = day % self.tracks.len();
        self.tracks.get(index)
    }
}

impl Command for SoundtrackDatabase
{
    fn apply(self, w: &mut World)
    {
        let to_load = self
            .tracks
            .iter()
            .map(|t| LoadedAudio { audio: t.source.clone(), ..default() })
            .collect();
        LoadAudio(to_load).apply(w);
        w.flush();
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct AudioPlugin;

impl Plugin for AudioPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<SoundtrackDatabase>()
            .register_command::<SoundtrackDatabase>()
            .add_systems(OnEnter(GameState::DayStart), set_soundtrack);
    }
}

//-------------------------------------------------------------------------------------------------------------------
