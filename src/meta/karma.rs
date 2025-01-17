use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn consolidate_karma(mut c: Commands, mut karma: ReactResMut<Karma>)
{
    karma.get_mut(&mut c).consolidate();
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactResource, Default, Debug)]
pub struct Karma
{
    /// Karma collected during the current day.
    day_collected: usize,
    /// Total Karma banked.
    total: usize,
}

impl Karma
{
    pub fn add(&mut self, karma: usize)
    {
        self.day_collected += karma;
    }

    pub fn consolidate(&mut self)
    {
        self.total += self.day_collected;
        self.day_collected = 0;
    }

    /// Tries to spend `amount`.
    pub fn spend(&mut self, amount: usize) -> bool
    {
        self.consolidate(); // in case of dev commands

        if amount > self.total {
            return false;
        }

        self.total -= amount;
        true
    }

    pub fn day_collected(&self) -> usize
    {
        self.day_collected
    }

    pub fn total(&self) -> usize
    {
        self.total + self.day_collected
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct KarmaPlugin;

impl Plugin for KarmaPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<Karma>()
            .add_systems(OnEnter(GameState::DayStart), consolidate_karma)
            // consolidate in case of dev commands that added karma
            .add_systems(OnEnter(GameState::Play), consolidate_karma);
    }
}

//-------------------------------------------------------------------------------------------------------------------
