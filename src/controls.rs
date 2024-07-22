use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Deref, DerefMut, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyCodeWrapper(KeyCode);

impl Default for KeyCodeWrapper
{
    fn default() -> Self
    {
        Self(KeyCode::KeyA)
    }
}

#[derive(ReactResource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Controls
{
    pub move_up: KeyCodeWrapper,
    pub move_down: KeyCodeWrapper,
    pub move_left: KeyCodeWrapper,
    pub move_right: KeyCodeWrapper,
}

impl Command for Controls
{
    fn apply(self, w: &mut World)
    {
        w.syscall(
            self,
            |In(new): In<Controls>, mut c: Commands, mut constants: ReactResMut<Controls>| {
                *constants.get_mut(&mut c) = new;
            },
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<Controls>()
            .init_react_resource::<Controls>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
