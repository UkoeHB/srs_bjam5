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

/// Types of power-up sources.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PowerUpSource
{
    #[default]
    LevelUp,
}

//-------------------------------------------------------------------------------------------------------------------

/// Coordinates collecting power-ups and handling them, in case multiple are collected at once.
#[derive(Resource, Default)]
pub struct BufferedPowerUps
{
    buffer: Vec<PowerUpSource>,
    is_handling: bool,
}

impl BufferedPowerUps
{
    pub fn insert(&mut self, additional: impl IntoIterator<Item = PowerUpSource>)
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

    pub fn current_powerup(&self) -> Option<PowerUpSource>
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

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PowerUpUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<BufferedPowerUps>()
            .add_systems(Update, handle_buffered_powerups.in_set(PowerUpUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
