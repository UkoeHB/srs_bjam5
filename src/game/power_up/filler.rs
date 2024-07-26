use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Filler power-ups when the player has maxed everything out.
#[derive(Debug, Copy, Clone)]
pub enum FillerType
{
    /// Recover % missing health.
    Health,
    /// Gain karma.
    Karma,
}

impl FillerType
{
    /// System that applies the filler type to the world.
    pub fn apply(
        In(filler_type): In<Self>,
        mut c: Commands,
        data: Res<FillerDatabase>,
        mut karma: ReactResMut<Karma>,
        mut player: Query<&mut Health, With<Player>>,
    )
    {
        match filler_type {
            Self::Health => {
                let Ok(mut hp) = player.get_single_mut() else { return };
                let missing = hp.max.saturating_sub(hp.current);
                let to_gain = ((missing as f32) * (data.get_amount(filler_type) as f32) / 100.).round() as usize;
                hp.add(to_gain);
            }
            Self::Karma => {
                karma.get_mut(&mut c).add(data.get_amount(filler_type));
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillerDatabase
{
    /// (name, description, amount)
    health: (String, String, usize),
    /// (name, description, amount)
    karma: (String, String, usize),
}

impl FillerDatabase
{
    pub fn get_info(&self, filler_type: FillerType) -> (&String, &String)
    {
        match filler_type {
            FillerType::Health => (&self.health.0, &self.health.1),
            FillerType::Karma => (&self.karma.0, &self.karma.1),
        }
    }

    pub fn get_amount(&self, filler_type: FillerType) -> usize
    {
        match filler_type {
            FillerType::Health => self.health.2,
            FillerType::Karma => self.karma.2,
        }
    }
}

impl Command for FillerDatabase
{
    fn apply(self, w: &mut World)
    {
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct FillerPlugin;

impl Plugin for FillerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<FillerDatabase>()
            .init_resource::<FillerDatabase>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
