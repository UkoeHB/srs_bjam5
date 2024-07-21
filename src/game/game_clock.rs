use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct GameClock
{
    pub elapsed: Duration,
}

impl GameClock
{
    pub fn elapsed_secs(&self) -> u64
    {
        self.elapsed.as_secs()
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn reset_game_clock(mut clock: ResMut<GameClock>)
{
    clock.elapsed = Duration::default();
}

//-------------------------------------------------------------------------------------------------------------------

fn update_game_clock(mut c: Commands, time: Res<Time<Virtual>>, mut clock: ResMut<GameClock>)
{
    let prev = clock.elapsed;
    clock.elapsed += time.delta();
    if prev.as_secs() < clock.elapsed.as_secs() {
        c.react().broadcast(GameClockIncremented);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct GameClockPlugin;

impl Plugin for GameClockPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<GameClock>()
            .add_systems(OnExit(GameState::DayOver), reset_game_clock)
            .add_systems(PreUpdate, update_game_clock.run_if(in_state(GameState::Play)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
