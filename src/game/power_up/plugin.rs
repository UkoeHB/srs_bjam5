use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PassivesUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct AbilitiesUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PowerUpActivateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(PowerupBankPlugin)
            .add_plugins(PlayerPowerupPlugin)
            .add_plugins(PowerupOptionsPlugin)
            .add_plugins(AbilitiesPlugin)
            .add_plugins(PassivesPlugin)
            .add_plugins(FillerPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
