use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct GamePlugin;

impl Plugin for GamePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(AnimationEventsPlugin)
            .add_plugins(BillboardCachePlugin)
            .add_plugins(DayEndPlugin)
            .add_plugins(SpriteLayersPlugin)
            .add_plugins(MapPlugin)
            .add_plugins(MobPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(CollectablesPlugin)
            .add_plugins(IntersectionsPlugin)
            .add_plugins(AttractionPlugin)
            .add_plugins(DamagePlugin)
            .add_plugins(PowerUpPlugin)
            .add_plugins(SpawningPlugin)
            .add_plugins(ProjectilePlugin)
            .add_plugins(StatsPlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(GameClockPlugin)
            .add_plugins(GameCameraPlugin)
            .add_plugins(LightPlugin)
            .configure_sets(
                Update,
                (
                    PassivesUpdateSet,
                    StatsUpdateSet,
                    PrevLocationUpdateSet,
                    PlayerUpdateSet,
                    MobUpdateSet,
                    CollectablesUpdateSet,
                    AttractionUpdateSet,
                    ProjectileUpdateSet,
                    AbilitiesUpdateSet,
                    EffectUpdateSet,
                    DamageUpdateSet,
                    MapConstraintsSet,
                    CameraUpdateSet,
                    PowerUpActivateSet,
                )
                    .chain()
                    .run_if(in_state(PlayState::Day)),
            )
            .add_effect_target::<Player>()
            .add_effect_target::<Mob>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
