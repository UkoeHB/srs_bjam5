use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(SpriteAnimationLoadPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
