use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct DayStartPlugin;

impl Plugin for DayStartPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(SettingsPlugin).add_plugins(StartUiPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
