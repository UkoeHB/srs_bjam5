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
            .add_plugins(MapPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(PowerUpPlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(GameClockPlugin)
            .add_plugins(GameCameraPlugin)
            .configure_sets(
                Update,
                (PlayerUpdateSet, CameraUpdateSet, PowerUpUpdateSet)
                    .chain()
                    .run_if(in_state(PlayState::Day)),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
