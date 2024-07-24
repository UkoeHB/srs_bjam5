use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct GamePlugin;

impl Plugin for GamePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BillboardCachePlugin)
            .add_plugins(GameSetupPlugin)
            .add_plugins(SpriteLayersPlugin)
            .add_plugins(MapPlugin)
            .add_plugins(MobPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(IntersectionsPlugin)
            .add_plugins(AttractionPlugin)
            .add_plugins(PowerUpPlugin)
            .add_plugins(SpawningPlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(GameClockPlugin)
            .add_plugins(GameCameraPlugin)
            .add_plugins(LightPlugin)
            .configure_sets(
                Update,
                (
                    PlayerUpdateSet,
                    AttractionUpdateSet,
                    IntersectionsUpdateSet,
                    CameraUpdateSet,
                    PowerUpUpdateSet,
                )
                    .chain()
                    .run_if(in_state(PlayState::Day)),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
