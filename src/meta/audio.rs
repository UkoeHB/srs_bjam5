use bevy::audio::{PlaybackMode, Volume};
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_volume(settings: ReactRes<AudioSettings>, mut bg: Query<(&mut AudioSink, &BackgroundAudio)>)
{
    let Ok((sink, bg)) = bg.get_single_mut() else { return };
    sink.set_volume(bg.volume * settings.master_volume);
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_audio(c: &mut Commands, master_volume: f32, soundtrack: &Soundtrack, audio: &AudioMap)
{
    c.spawn((
        AudioBundle {
            source: audio.get(&soundtrack.source),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(soundtrack.volume * master_volume),
                ..default()
            },
            ..default()
        },
        BackgroundAudio(soundtrack.clone()),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

fn set_soundtrack(
    mut c: Commands,
    soundtracks: Res<SoundtrackDatabase>,
    audio: Res<AudioMap>,
    settings: ReactRes<AudioSettings>,
    day: ReactRes<Day>,
    query: Query<(Entity, &BackgroundAudio)>,
)
{
    let Some(current_track) = soundtracks.get(day.get()) else { return };
    let Ok((entity, background)) = query.get_single() else {
        spawn_audio(&mut c, settings.master_volume, current_track, &audio);
        return;
    };
    if background.source == current_track.source {
        return;
    }
    c.entity(entity).despawn_recursive();
    spawn_audio(&mut c, settings.master_volume, current_track, &audio);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Deref)]
pub struct BackgroundAudio(Soundtrack);

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
        let index = std::cmp::min(day.saturating_sub(1), self.tracks.len());
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

#[derive(ReactResource, Debug)]
pub struct AudioSettings
{
    pub master_volume: f32,
}

//-------------------------------------------------------------------------------------------------------------------

pub struct AudioPlugin;

impl Plugin for AudioPlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_react_resource(AudioSettings { master_volume: 1.0 })
            .init_resource::<SoundtrackDatabase>()
            .register_command::<SoundtrackDatabase>()
            .add_systems(OnEnter(GameState::DayStart), set_soundtrack)
            .react(|rc| rc.on_persistent(resource_mutation::<AudioSettings>(), update_volume));
    }
}

//-------------------------------------------------------------------------------------------------------------------
