use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

// todo: game map
// todo: display controls on ground at starting location

//-------------------------------------------------------------------------------------------------------------------

fn reset_game(mut c: Commands, sounds: Query<Entity, With<Handle<AudioSource>>>)
{
    for entity in sounds.iter() {
        c.entity(entity).despawn_recursive();
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(GameState::Play), reset_game);
    }
}

//-------------------------------------------------------------------------------------------------------------------
