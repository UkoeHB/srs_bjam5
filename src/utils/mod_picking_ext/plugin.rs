use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct ModPickingExtPlugin;

impl Plugin for ModPickingExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(PickingInteractionExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
