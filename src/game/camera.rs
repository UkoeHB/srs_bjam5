use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_cobweb::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_camera(
    constants: ReactRes<GameConstants>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut Transform), With<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
    bg: Query<(&TilemapGridSize, &TilemapType), With<BackgroundTilemap>>,
    player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
)
{
    let (camera, cam_global, mut camera_transform) = camera.single_mut();
    let window = window.single();
    let (bg_grid_size, bg_tilemap_type) = bg.single();
    let player_transform = player.single();

    // Get starting position of camera.
    let cam_translation = &mut camera_transform.translation;
    let (cam_lower_left, cam_upper_right) = get_camera_corners(&camera, &cam_global, &window);

    // Get boundaries of map.
    let tile_radius = constants.map_tile_size.x / 2.;
    let upper_right = TilePos { x: constants.map_size.x - 1, y: constants.map_size.y - 1 };
    let map_upper_right =
        upper_right.center_in_world(bg_grid_size, bg_tilemap_type) / 2. + Vec2::splat(tile_radius);
    let map_lower_left = -map_upper_right;

    // Get translation so camera will center on player.
    let mut to_translate = player_transform.translation - *cam_translation;

    // Correct the translation so the camera edges stay within the map.
    to_translate.x -= ((cam_upper_right.x + to_translate.x) - map_upper_right.x).max(0.); // right edge
    to_translate.x -= ((cam_lower_left.x + to_translate.x) - map_lower_left.x).min(0.); // left edge
    to_translate.y -= ((cam_upper_right.y + to_translate.y) - map_upper_right.y).max(0.); // top edge
    to_translate.y -= ((cam_lower_left.y + to_translate.y) - map_lower_left.y).min(0.); // bottom edge

    // Save result
    *cam_translation += to_translate;
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `(lower left, upper right)` corners.
pub fn get_camera_corners(camera: &Camera, cam_global: &GlobalTransform, window: &Window) -> (Vec2, Vec2)
{
    let window_dims = Vec2 { x: window.width(), y: window.height() };

    let cam_upper_right = camera
        .viewport_to_world_2d(cam_global, Vec2 { x: window_dims.x, y: 0. })
        .unwrap();
    let cam_lower_left = camera
        .viewport_to_world_2d(cam_global, Vec2 { x: 0., y: window_dims.y })
        .unwrap();

    (cam_lower_left, cam_upper_right)
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
