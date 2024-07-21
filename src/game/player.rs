use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Player
{
    pub health: usize,
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_player(mut c: Commands, constants: ReactRes<GameConstants>)
{
    //todo: scoping to GameState::Play means the player despawns on entering GameState::DayOver, even though we
    // may want to continue displaying the player in the background
    c.spawn((
        Player { health: constants.player_base_hp },
        StateScoped(GameState::Play),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(GameState::Play), spawn_player);
    }
}

//-------------------------------------------------------------------------------------------------------------------
