//! Components shared by different kinds of entities.

use bevy::prelude::*;

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

    pub fn set_health(&mut self, new: usize)
    {
        self.current = new.min(self.max)
    }
}

//-------------------------------------------------------------------------------------------------------------------
