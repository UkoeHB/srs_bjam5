use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

// todo: game map
// todo: display controls on ground at starting location

//-------------------------------------------------------------------------------------------------------------------

// Forwards game day end conditions as GameDayOver.
fn send_day_over(mut c: Commands)
{
    c.react().broadcast(GameDayOver);
}

//-------------------------------------------------------------------------------------------------------------------

fn check_day_end_condition(mut c: Commands, constants: ReactRes<GameConstants>, game_clock: Res<GameClock>)
{
    // Condition: time ran out
    if game_clock.elapsed_secs() >= constants.day_length_secs {
        c.react().broadcast(PlayerSurvived);
    }
}

//--------------------------------------------------------------------------------------------------------------------

fn check_entity_health(mut c: Commands, entities: Query<(Entity, &Health), Changed<Health>>)
{
    for (id, health) in entities.iter() {
        // dead if health is 0 (can't be less)
        if health.current == 0 {
            c.trigger_targets(EntityDeath, id);
            // removes component because otherwise it would keep detecting it as dead
            c.entity(id).remove::<Health>();
        }
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
        });
        app.add_systems(
            Update,
            (check_entity_health, check_day_end_condition).run_if(in_state(PlayState::Day)),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
