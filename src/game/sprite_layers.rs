use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin};

//-------------------------------------------------------------------------------------------------------------------

//todo: sprite layer ordering is NOT compatible with bevy_mod_picking order
// (see https://github.com/deifactor/extol_sprite_layer/issues/6)
#[derive(Debug, Copy, Clone, Component, PartialEq, Eq, Hash)]
pub enum SpriteLayer
{
    Background,
    BackgroundBillboard,
    GroundEffect,
    DyingEnemy,
    /// Includes: player, enemies, player and enemy projectiles
    Objects,
}

impl LayerIndex for SpriteLayer
{
    fn as_z_coordinate(&self) -> f32
    {
        use SpriteLayer::*;
        match *self {
            Background => 0.,
            BackgroundBillboard => 1.,
            GroundEffect => 2.,
            DyingEnemy => 3.,
            Objects => 4.,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SpriteLayersPlugin;

impl Plugin for SpriteLayersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(SpriteLayerPlugin::<SpriteLayer>::default());
    }
}

//-------------------------------------------------------------------------------------------------------------------
