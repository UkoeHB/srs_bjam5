//! Components representing player and mob attributes.

use std::time::Duration;

use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn apply_health_regen(
    mut next: Local<Duration>,
    clock: Res<GameClock>,
    mut regen: Query<(&HealthRegen, &mut Health)>,
)
{
    if clock.elapsed < *next {
        return;
    }
    *next += Duration::from_secs(1);

    for (regen, mut health) in regen.iter_mut() {
        health.add(regen.current());
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct Health
{
    current: usize,
    base_max: usize,
    bonus: usize,
}

impl Health
{
    pub fn new(base_max: usize) -> Self
    {
        Self { current: base_max, base_max, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.current
    }

    pub fn missing(&self) -> usize
    {
        self.max().saturating_sub(self.current())
    }

    pub fn max(&self) -> usize
    {
        self.base_max + self.bonus
    }

    pub fn add(&mut self, add: usize)
    {
        self.current += add;
        self.current = self.current.min(self.max());
    }

    pub fn remove(&mut self, sub: usize)
    {
        self.current = self.current.saturating_sub(sub);
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        // When increasing max hp, add to current health.
        let diff = bonus.saturating_sub(self.bonus);
        self.bonus = bonus;
        self.add(diff);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Regenerates health every second on the second.
#[derive(Debug, Component)]
pub struct HealthRegen
{
    base: usize,
    bonus: usize,
}

impl HealthRegen
{
    pub fn new(base: usize) -> Self
    {
        Self { base, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.base + self.bonus
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct Armor
{
    base: usize,
    bonus: usize,
}

impl Armor
{
    pub fn new(base: usize) -> Self
    {
        Self { base, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.base + self.bonus
    }

    pub fn calculate_damage(&self, damage: f32) -> f32
    {
        damage * 100. / (self.current() as f32 + 100.)
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct CooldownReduction
{
    base: usize,
    bonus: usize,
}

impl CooldownReduction
{
    pub fn new(base: usize) -> Self
    {
        Self { base, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.base + self.bonus
    }

    pub fn calculate_cooldown(&self, cooldown: u64) -> u64
    {
        ((cooldown as f32) * 100. / (self.current() as f32 + 100.)).round() as u64
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct MoveSpeed
{
    base: usize,
    bonus: usize,
}

impl MoveSpeed
{
    pub fn new(base: usize) -> Self
    {
        Self { base, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.base + self.bonus
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct CollectionRange
{
    base: usize,
    bonus: usize,
}

impl CollectionRange
{
    pub fn new(base: usize) -> Self
    {
        Self { base, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.base + self.bonus
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// area_size * base_area = area
#[derive(Debug, Component)]
pub struct AreaSize
{
    base: f32,
    bonus: f32,
}

impl AreaSize
{
    pub fn new(base: f32) -> Self
    {
        Self { base, bonus: 0. }
    }

    pub fn current(&self) -> f32
    {
        self.base + self.bonus
    }

    pub fn calculate_area(&self, area: Vec2) -> Vec2
    {
        area * self.current()
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = (bonus as f32) / 100.;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct DamageAmp
{
    base: usize,
    bonus: usize,
}

impl DamageAmp
{
    pub fn new(base: usize) -> Self
    {
        Self { base, bonus: 0 }
    }

    pub fn current(&self) -> usize
    {
        self.base + self.bonus
    }

    pub fn calculate_damage(&self, damage: f32) -> f32
    {
        damage + damage * (self.current() as f32) / 100.
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
pub struct ExpAmp
{
    base: f32,
    bonus: f32,
}

impl ExpAmp
{
    pub fn new(base: usize) -> Self
    {
        Self { base: base as f32, bonus: 0. }
    }

    pub fn current(&self) -> f32
    {
        self.base + self.bonus
    }

    pub fn calculate_exp(&self, exp: f32) -> f32
    {
        exp + exp * self.current() / 100.
    }

    pub fn set_bonus(&mut self, bonus: usize)
    {
        self.bonus = bonus as f32;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Level
{
    level: usize,
    exp: f32,

    starting_exp_req: f32,
    /// Additional exp required per level.
    //todo: currently linear, more sophisticated?
    exp_gain_rate: f32,
}

impl Level
{
    pub fn new(starting_exp_req: usize, exp_gain_rate: usize) -> Self
    {
        Self {
            level: 1,
            exp: 0.,
            starting_exp_req: starting_exp_req as f32,
            exp_gain_rate: exp_gain_rate as f32,
        }
    }

    /// Returns a vec of newly gained levels.
    #[must_use]
    pub fn add_exp(&mut self, exp: usize, amp: &ExpAmp) -> Vec<usize>
    {
        self.exp += amp.calculate_exp(exp as f32);

        let mut levels = Vec::default();
        while self.exp >= self.exp_required() {
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
        self.exp.round() as usize
    }

    pub fn exp_required(&self) -> f32
    {
        self.starting_exp_req + ((self.level - 1) as f32) * self.exp_gain_rate
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct StatsUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct StatsPlugin;

impl Plugin for StatsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, apply_health_regen.in_set(StatsUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
