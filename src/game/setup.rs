use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

// todo: game map
// todo: display controls on ground at starting location

//-------------------------------------------------------------------------------------------------------------------

/// Forwards game day end conditions as GameDayOver.
fn send_day_over(mut c: Commands)
{
    c.react().broadcast(GameDayOver);
}

//-------------------------------------------------------------------------------------------------------------------

//todo: use state scoped entities for audio instead?
fn reset_game(mut c: Commands, sounds: Query<Entity, With<Handle<AudioSource>>>)
{
    for entity in sounds.iter() {
        c.entity(entity).despawn_recursive();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn check_end_condition(
    mut c: Commands,
    player: Query<&Player>,
    constants: ReactRes<GameConstants>,
    game_clock: Res<GameClock>,
)
{
    // Condition: time ran out
    if game_clock.elapsed_secs() >= constants.day_length_secs {
        c.react().broadcast(PlayerSurvived);
        return;
    }

    // Condition: player health
    if player.single().health == 0 {
        c.react().broadcast(PlayerDied);
        return;
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
        .add_systems(Update, check_end_condition.run_if(in_state(GameState::Play)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
