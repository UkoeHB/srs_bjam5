use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Item that can be collected by the player.
#[derive(Component)]
pub enum Collectable
{
    Exp(usize),
    Karma(usize),
    HealthPack(usize),
}

impl Collectable
{
    pub fn get_detection_range(&self, constants: &GameConstants, _size: Vec2) -> Option<Vec2>
    {
        match self {
            Self::Exp(..) => Some(constants.exp_detection_range),
            Self::Karma(..) => None,
            Self::HealthPack(..) => None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------