use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PowerUpUpdateSet;

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
            .add_plugins(FillerPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
