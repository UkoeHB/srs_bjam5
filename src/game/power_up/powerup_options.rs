use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_buffered_powerups(mut c: Commands, mut buffer: ResMut<BufferedPowerUps>)
{
    if buffer.is_handling_powerup() {
        return;
    }

    if !buffer.try_start_handling() {
        return;
    }

    c.react().broadcast(PlayerPowerUp);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn get_powerup_options(
    _rng: &mut GameRng,
    _source: PowerupSource,
    _player_powerups: &PlayerPowerups,
    _powerup_bank: &PowerupBank,
) -> Vec<PowerupOption>
{
    // todo:
    // - on level-up, at minimum 1 option should be 'new' if there are open slots; other slots are selected at
    // random proportional to number of open slots / total slots
    vec![PowerupOption::Filler(FillerType::Health), PowerupOption::Filler(FillerType::Karma)]
}

//-------------------------------------------------------------------------------------------------------------------

/// Types of power-up sources.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PowerupSource
{
    #[default]
    LevelUp,
}

//-------------------------------------------------------------------------------------------------------------------

/// Coordinates collecting power-ups and handling them, in case multiple are collected at once.
#[derive(Resource, Default)]
pub struct BufferedPowerUps
{
    buffer: Vec<PowerupSource>,
    is_handling: bool,
}

impl BufferedPowerUps
{
    pub fn insert(&mut self, additional: impl IntoIterator<Item = PowerupSource>)
    {
        self.buffer.extend(additional);
    }

    pub fn is_handling_powerup(&self) -> bool
    {
        self.is_handling
    }

    pub fn try_start_handling(&mut self) -> bool
    {
        if self.buffer.len() == 0 {
            return false;
        }
        self.is_handling = true;
        true
    }

    pub fn current_powerup(&self) -> Option<PowerupSource>
    {
        self.buffer.get(0).cloned()
    }

    pub fn end_handling_powerup(&mut self)
    {
        if !self.is_handling {
            return;
        }
        debug_assert!(self.buffer.len() > 0);
        if self.buffer.len() > 0 {
            self.buffer.remove(0);
        }
        self.is_handling = false;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum PowerupOption
{
    PowerUp,
    Filler(FillerType),
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PowerUpActivateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct PowerupOptionsPlugin;

impl Plugin for PowerupOptionsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<BufferedPowerUps>()
            .add_systems(Update, handle_buffered_powerups.in_set(PowerUpActivateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
