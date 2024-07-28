use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BeerCanPlugin)
            .add_plugins(NailGunPlugin)
            .add_plugins(CarBatteryPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
