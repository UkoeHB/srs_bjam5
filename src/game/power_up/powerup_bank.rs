use std::collections::HashMap;

use bevy::prelude::*;

//use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum AbilityType
{
    #[default]
    Passive,
    Active,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct PowerupInfo
{
    pub ability_type: AbilityType,
    pub name: String,
    pub description: String,
    /// Stores a string pointing to a spritesheet animation. We will use the first frame of that animation as the
    /// icon.
    pub icon: String,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Deref, Debug, Default)]
pub struct PowerupBank(HashMap<String, PowerupInfo>);

impl PowerupBank
{
    pub fn register(&mut self, info: PowerupInfo)
    {
        self.0.insert(info.name.clone(), info);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct PowerupBankPlugin;

impl Plugin for PowerupBankPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<PowerupBank>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
