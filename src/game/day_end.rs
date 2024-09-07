use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_day_over(mut c: Commands)
{
    c.set_state(PlayState::DayOver);
}

//-------------------------------------------------------------------------------------------------------------------

fn check_day_end_condition(mut c: Commands, constants: ReactRes<GameConstants>, game_clock: Res<GameClock>)
{
    // Condition: time ran out
    if game_clock.elapsed_secs() >= constants.day_length_secs {
        c.react().broadcast(PlayerSurvived);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct DayEndPlugin;

impl Plugin for DayEndPlugin
{
    fn build(&self, app: &mut App)
    {
        app.react(|rc| {
            rc.on_persistent(
                (broadcast::<PlayerDied>(), broadcast::<PlayerSurvived>()),
                handle_day_over,
            )
        });
        //todo: this races with the game clock update, need to use ordered system sets
        app.add_systems(PreUpdate, check_day_end_condition.run_if(in_state(PlayState::Day)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
