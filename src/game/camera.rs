use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_camera(
    mut camera: Query<&mut Transform, With<MainCamera>>,
    player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
)
{
    let mut camera_transform = camera.single_mut();
    let player_transform = player.single();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct CameraUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, (update_camera,).chain().in_set(CameraUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
