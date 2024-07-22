use core::f32::consts::PI;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn get_tile_index(constants: &GameConstants, selection: f32) -> usize
{
    let mut cumulative = 0.;
    let tile_configs = &constants.background_tile_configs;
    for (idx, tile) in tile_configs.iter().enumerate() {
        cumulative += tile.frequency;

        if cumulative > selection {
            return idx;
        }
    }

    debug_assert!(tile_configs.len() > 0);
    tile_configs.len() - 1
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_map(mut c: Commands, mut rng: ResMut<GameRng>, constants: ReactRes<GameConstants>, images: Res<ImageMap>)
{
    let rng = rng.rng();

    let tiles_texture = images.get(&constants.background_tile_texture);
    let tilemap_size = constants.map_size.into();
    let tilemap_tile_size: TilemapTileSize = constants.map_tile_size.into();

    // Background tilemap entity.
    let bg_tilemap_entity = c
        .spawn((BackgroundTilemap, StateScoped(GameState::Play)))
        .id();

    // Spawn the elements of the tilemap.
    let mut tile_storage = TileStorage::empty(tilemap_size);
    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos { x, y };

            let texture_selection = rng.gen_range((0.)..(1.));
            let texture_index = get_tile_index(&constants, texture_selection);

            let tile_entity = c
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(bg_tilemap_entity),
                        texture_index: TileTextureIndex(texture_index as u32),
                        ..Default::default()
                    },
                    StateScoped(GameState::Play),
                    SpriteLayer::Background,
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let grid_size = tilemap_tile_size.into();
    let map_type = TilemapType::default();

    c.entity(bg_tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tilemap_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tiles_texture),
        tile_size: tilemap_tile_size,
        transform: get_tilemap_center_transform(&tilemap_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn setup_map_boundary(
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    images: Res<ImageMap>,
    bg: Query<(&TilemapGridSize, &TilemapType), With<BackgroundTilemap>>,
)
{
    let (bg_grid_size, bg_tilemap_type) = bg.single();

    let side_img = images.get(&constants.boundary_side_texture);
    let corner_img = images.get(&constants.boundary_corner_texture);
    let map_size = constants.map_size;
    let vertical_sz = Vec2 { x: constants.boundary_width, y: constants.boundary_length };
    let horizontal_sz = Vec2 { x: constants.boundary_length, y: constants.boundary_width };

    // Correction to tile placement.
    // - Tile positions are calculated relative to first tile, NOT the transform origin which aligns with the
    //   center tile.
    let upper_right = TilePos { x: map_size.x - 1, y: map_size.y - 1 };
    let tile_correction = upper_right.center_in_world(bg_grid_size, bg_tilemap_type) / 2.;

    let add_tile = |c: &mut Commands, texture: Handle<Image>, x: u32, y: u32, size: Vec2, rotation: f32| {
        let tile_pos = TilePos { x, y };
        let bg_translation = tile_pos.center_in_world(bg_grid_size, bg_tilemap_type) - tile_correction;
        let tile_translation = bg_translation.extend(0.0);
        let mut transform = Transform::from_translation(tile_translation);
        transform.rotate_z(rotation);

        c.spawn((
            SpriteBundle { texture, transform, ..default() },
            Barrier,
            AabbSize(size),
            SpriteLayer::Objects,
            StateScoped(GameState::Play),
        ));
    };

    let add_corner_extra_bounding_box = |c: &mut Commands, x: u32, y: u32, size: Vec2| {
        let tile_pos = TilePos { x, y };
        let bg_translation = tile_pos.center_in_world(bg_grid_size, bg_tilemap_type) - tile_correction;
        let tile_translation = bg_translation.extend(0.0);
        let transform = Transform::from_translation(tile_translation);

        c.spawn((
            SpatialBundle { transform, ..default() },
            Barrier,
            AabbSize(size),
            SpriteLayer::Objects,
            StateScoped(GameState::Play),
        ));
    };

    // Top and bottom
    for x in 1..(map_size.x - 1) {
        for y in [0, map_size.y - 1] {
            add_tile(&mut c, side_img.clone(), x, y, horizontal_sz, FRAC_PI_2);
        }
    }

    // Left and right
    for y in 1..(map_size.y - 1) {
        for x in [0, map_size.x - 1] {
            add_tile(&mut c, side_img.clone(), x, y, vertical_sz, 0.);
        }
    }

    // Bottom-right
    add_tile(&mut c, corner_img.clone(), 0, 0, horizontal_sz, FRAC_PI_2);
    add_corner_extra_bounding_box(&mut c, 0, 0, vertical_sz);

    // Bottom-left
    add_tile(&mut c, corner_img.clone(), map_size.x - 1, 0, horizontal_sz, PI);
    add_corner_extra_bounding_box(&mut c, map_size.x - 1, 0, vertical_sz);

    // Top-left
    add_tile(
        &mut c,
        corner_img.clone(),
        map_size.x - 1,
        map_size.y - 1,
        horizontal_sz,
        -FRAC_PI_2,
    );
    add_corner_extra_bounding_box(&mut c, map_size.x - 1, map_size.y - 1, vertical_sz);

    // Top-right
    add_tile(&mut c, corner_img.clone(), 0, map_size.y - 1, horizontal_sz, 0.);
    add_corner_extra_bounding_box(&mut c, 0, map_size.y - 1, vertical_sz);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn map_area_size(constants: &GameConstants) -> Vec2
{
    Vec2 {
        x: (((constants.map_size.x as f32) - 1.) * constants.map_tile_size.x) - constants.boundary_width,
        y: (((constants.map_size.y as f32) - 1.) * constants.map_tile_size.y) - constants.boundary_width,
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn map_area_half_size(constants: &GameConstants) -> Vec2
{
    let map_area_size = map_area_size(constants);
    Vec2 { x: map_area_size.x / 2., y: map_area_size.y / 2. }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the tilemap that controls the background.
#[derive(Component)]
pub struct BackgroundTilemap;

//-------------------------------------------------------------------------------------------------------------------

/// Configures how often a tile within a spritesheet of background tiles should be selected.
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileConfig
{
    pub frequency: f32,
}

//-------------------------------------------------------------------------------------------------------------------

pub struct MapPlugin;

impl Plugin for MapPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(GameState::Play), (spawn_map, setup_map_boundary).chain());
    }
}

//-------------------------------------------------------------------------------------------------------------------
