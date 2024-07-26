use std::collections::HashMap;

use bevy::prelude::*;

//use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub enum PowerupType
{
    #[default]
    Passive,
    Active,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct PowerupInfo
{
    pub powerup_type: PowerupType,
    pub name: String,
    pub description: String,
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
