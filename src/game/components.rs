//! Components shared by different kinds of entities.

use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// An entity that mobile entities can't move through.
#[derive(Component)]
pub struct Barrier;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Event)]
pub struct EntityDeath;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct Health
{
    pub current: usize,
    pub max: usize,
}

impl Health
{
    pub fn from_max(max: usize) -> Self
    {
        Self { current: max, max }
    }

    pub fn set_current(&mut self, new: usize)
    {
        self.current = new.min(self.max)
    }

    pub fn add(&mut self, add: usize)
    {
        self.current += add;
        self.current = self.current.max(self.max);
    }

    pub fn remove(&mut self, sub: usize)
    {
        self.current = self.current.saturating_sub(sub);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Armor
{
    pub armor: usize,
}

impl Armor
{
    pub fn new(armor: usize) -> Self
    {
        Self { armor }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Level
{
    level: usize,
    exp: usize,

    starting_exp_req: usize,
    /// Additional exp required per level.
    //todo: currently linear, more sophisticated?
    exp_gain_rate: usize,
}

impl Level
{
    pub fn new(starting_exp_req: usize, exp_gain_rate: usize) -> Self
    {
        Self { level: 1, exp: 0, starting_exp_req, exp_gain_rate }
    }

    /// Returns a vec of newly gained levels.
    #[must_use]
    pub fn add_exp(&mut self, exp: usize) -> Vec<usize>
    {
        self.exp += exp;

        let mut levels = Vec::default();
        while self.exp > self.exp_required() {
            self.exp -= self.exp_required();
            self.level += 1;
            levels.push(self.level);
        }
        levels
    }

    pub fn level(&self) -> usize
    {
        self.level
    }

    pub fn exp(&self) -> usize
    {
        self.exp
    }

    pub fn exp_required(&self) -> usize
    {
        self.starting_exp_req + (self.level - 1) * self.exp_gain_rate
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_collider_effect_impl(
    In((collider, target)): In<(Entity, Entity)>,
    mut events: EventWriter<DamageEvent>,
    colliders: Query<&Collider>,
)
{
    let Ok(collider) = colliders.get(collider) else { return };
    events.send(DamageEvent { target, damage: collider.damage });
}

pub fn apply_collider_effect(collider: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((collider, target), apply_collider_effect_impl);
}

/// Component for collider effects.
#[derive(Component, Debug)]
pub struct Collider
{
    pub damage: usize,
}

//-------------------------------------------------------------------------------------------------------------------

/// Component for emitter mobs.
#[derive(Component, Debug)]
pub struct Emitter
{
    cooldown_ms: u64,
    projectile: ProjectileConfig,

    next_fire_time: Option<Duration>,
}

impl Emitter
{
    pub fn new(cooldown_ms: u64, projectile: ProjectileConfig) -> Self
    {
        Self { cooldown_ms, projectile, next_fire_time: None }
    }

    pub fn config(&self) -> &ProjectileConfig
    {
        &self.projectile
    }

    /// Returns true if the emitter should fire.
    pub fn update_cooldown(&mut self, time: Duration) -> bool
    {
        if let Some(next) = self.next_fire_time {
            if time < next {
                return false;
            }
        }

        self.next_fire_time = Some(time + Duration::from_millis(self.cooldown_ms));
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------
