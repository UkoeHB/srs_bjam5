use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct GamePlugin;

impl Plugin for GamePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(GameSetupPlugin)
            .add_plugins(SpriteLayersPlugin)
            .add_plugins(GameUiPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
