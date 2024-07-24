use std::collections::HashMap;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MobOnDeathType
{
    Explode
    {
        base_damage: usize
    },
}

impl Default for MobOnDeathType
{
    fn default() -> Self
    {
        Self::Explode { base_damage: 0 }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MobType
{
    Collider
    {
        base_damage: usize,
        base_cooldown_millis: u64,
    },
    Emitter
    {
        base_damage: usize,
        base_cooldown_millis: u64,
        /// Range in transform units.
        base_fire_range: f32,
    },
    OnDeath(MobOnDeathType),
}

impl Default for MobType
{
    fn default() -> Self
    {
        Self::Collider { base_damage: 0, base_cooldown_millis: 1000 }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MobData
{
    pub animation: String,
    /// This should usually equal the animation size, but doesn't have to.
    pub hitbox: Vec2,
    pub base_health: usize,
    pub base_armor: usize,
    pub base_speed_tps: f32,
    pub mob_type: MobType,
}

//-------------------------------------------------------------------------------------------------------------------

/// [mob name, mob data]
#[derive(Resource, Deref, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MobDatabase(HashMap<String, MobData>);

impl Command for MobDatabase
{
    fn apply(self, w: &mut World)
    {
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct MobPlugin;

impl Plugin for MobPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<MobDatabase>()
            .init_resource::<MobDatabase>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
