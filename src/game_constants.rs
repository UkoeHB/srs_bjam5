use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactResource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConstants
{
    pub day_length_secs: u64,

    pub spawn_point_cadence_secs: usize,
    /// Additional space to add to spawn circle.
    pub spawn_radius_buffer: f32,
    /// Half-size of square around spawn location where spawns are randomly placed.
    pub spawn_adjustment_size: f32,

    pub collectable_max_vel: f32,
    pub collectable_accel: f32,

    pub exp_detection_range: Vec2,

    pub player_size: Vec2,
    pub player_standing_animation: String,

    /// Player run speed in transform units per second.
    pub player_run_speed_tps: f32,
    pub player_base_hp: usize,
    pub player_exp_start: usize,
    pub player_exp_rate: usize,

    pub level_tag_font: String,
    pub level_tag_color: Color,
    pub level_tag_font_size: f32,
    pub level_tag_offset: Vec2,

    pub exp_bar_size: Vec2,
    pub exp_bar_offset: f32,
    pub exp_bar_filled_color: Color,
    pub exp_bar_empty_color: Color,

    pub hp_bar_size: Vec2,
    pub hp_bar_offset: f32,
    pub hp_bar_filled_color: Color,
    pub hp_bar_empty_color: Color,

    pub background_tile_texture: String,
    pub background_tile_configs: Vec<TileConfig>,
    /// The total map area in number of tiles (rectangular). todo: consider making this programmatic per-day?
    pub map_size: UVec2,
    /// The size of each map tile.
    pub map_tile_size: Vec2,

    pub controls_texture: String,

    pub boundary_side_texture: String,
    pub boundary_corner_texture: String,
    pub boundary_width: f32,
    pub boundary_length: f32,
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
