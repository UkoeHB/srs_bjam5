use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct MetaPlugin;

impl Plugin for MetaPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(DayPlugin)
            .add_plugins(KarmaPlugin)
            .add_plugins(AudioPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
