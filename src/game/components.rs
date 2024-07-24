//! Components shared by different kinds of entities.

use bevy::prelude::*;
//use bevy_cobweb::prelude::*;

//use crate::*;

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

    pub fn subtract(&mut self, sub: usize)
    {
        self.current = self.current.saturating_sub(sub);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Component)]
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
