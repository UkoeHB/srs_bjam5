use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

// todo: game map
// todo: display controls on ground at starting location

//-------------------------------------------------------------------------------------------------------------------

fn send_day_over(mut c: Commands)
{
    c.react().broadcast(GameDayOver);
}

//-------------------------------------------------------------------------------------------------------------------

fn reset_game(mut c: Commands, sounds: Query<Entity, With<Handle<AudioSource>>>)
{
    for entity in sounds.iter() {
        c.entity(entity).despawn_recursive();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn check_death_condition(mut c: Commands, constants: ReactRes<GameConstants>)
{
    // todo: actually check the death condition
    if constants.player_base_hp > 0 {
        c.react().broadcast(PlayerSurvived);
    } else {
        c.react().broadcast(PlayerDied);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.react(|rc| {
            rc.on_persistent(
                (broadcast::<PlayerDied>(), broadcast::<PlayerSurvived>()),
                send_day_over,
            )
        })
        .add_systems(OnEnter(GameState::Play), reset_game)
        .add_systems(Update, check_death_condition.run_if(in_state(GameState::Play)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
