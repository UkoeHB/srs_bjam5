use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactResource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConstants
{
    pub day_length_secs: usize,
    pub player_base_hp: usize,
}

impl Command for GameConstants
{
    fn apply(self, w: &mut World)
    {
        w.syscall(
            self,
            |In(new): In<GameConstants>, mut c: Commands, mut constants: ReactResMut<GameConstants>| {
                *constants.get_mut(&mut c) = new;
            },
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct GameConstantsPlugin;

impl Plugin for GameConstantsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<GameConstants>()
            .init_react_resource::<GameConstants>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
