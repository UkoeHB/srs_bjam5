
pub fn setup_map(mut commands: Commands, mut rng: ResMut<GameRng>, asset_server: Res<AssetServer>)
{
    let rng = rng.rng();

    let tiles_texture = load_tile_assets(&asset_server);
    let tilemap_size = TILEMAP_SIZE;

    // Background tilemap entity.
    let bg_tilemap_entity = commands.spawn((BackgroundTilemap, Clearable)).id();

    // Spawn the elements of the tilemap.
    let mut tile_storage = TileStorage::empty(tilemap_size);
    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos { x, y };

            let texture_selection = rng.gen_range((0.)..(1.));
            let texture_index = get_tile_index(texture_selection);

            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(bg_tilemap_entity),
                        texture_index: TileTextureIndex(texture_index as u32),
                        ..Default::default()
                    },
                    Clearable,
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TILEMAP_TILE_SIZE;
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(bg_tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tilemap_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tiles_texture),
        tile_size,
        transform: get_tilemap_center_transform(&tilemap_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

pub fn setup_map_boundary(
    mut cmds: Commands,
    config: Res<GameConfig>,
    asset_server: Res<AssetServer>,
    bg: Query<(&TilemapGridSize, &TilemapType), With<BackgroundTilemap>>,
)
{
    let (bg_grid_size, bg_tilemap_type) = bg.single();

    let side_img: Handle<Image> = asset_server.load(BOUNDARY_SIDE_IMG);
    let corner_img: Handle<Image> = asset_server.load(BOUNDARY_CORNER_IMG);
    let map_size = config.map_size;
    let vertical_sz = BOUNDARY_SIZE_VERTICAL;
    let horizontal_sz = Vec2 { x: BOUNDARY_SIZE_VERTICAL.y, y: BOUNDARY_SIZE_VERTICAL.x };

    // Correction to tile placement.
    // - Tile positions are calculated relative to the origin, not the map's origin...
    let upper_left = TilePos { x: map_size.x - 1, y: map_size.y - 1 };
    let tile_correction = upper_left.center_in_world(bg_grid_size, bg_tilemap_type) / 2.;

    let add_tile = |cmds: &mut Commands, texture: Handle<Image>, x: u32, y: u32, size: Vec2, rotation: f32| {
        let tile_pos = TilePos { x, y };
        let bg_translation = tile_pos.center_in_world(bg_grid_size, bg_tilemap_type) - tile_correction;
        let tile_translation = bg_translation.extend(BOUNDARY_VERTICAL_POSITION);
        let mut transform = Transform::from_translation(tile_translation);
        transform.rotate_z(rotation);

        cmds.spawn((
            SpriteBundle { texture, transform, ..default() },
            Barrier,
            AabbSize(size),
            Clearable,
        ));
    };

    let add_corner_extra_bounding_box = |cmds: &mut Commands, x: u32, y: u32, size: Vec2| {
        let tile_pos = TilePos { x, y };
        let bg_translation = tile_pos.center_in_world(bg_grid_size, bg_tilemap_type) - tile_correction;
        let tile_translation = bg_translation.extend(BOUNDARY_VERTICAL_POSITION);
        let transform = Transform::from_translation(tile_translation);

        cmds.spawn((
            SpatialBundle { transform, ..default() },
            Barrier,
            AabbSize(size),
            Clearable,
        ));
    };

    // Top and bottom
    for x in 1..(map_size.x - 1) {
        for y in [0, map_size.y - 1] {
            add_tile(&mut cmds, side_img.clone(), x, y, horizontal_sz, FRAC_PI_2);
        }
    }

    // Left and right
    for y in 1..(map_size.y - 1) {
        for x in [0, map_size.x - 1] {
            add_tile(&mut cmds, side_img.clone(), x, y, vertical_sz, 0.);
        }
    }

    // Bottom-right
    add_tile(&mut cmds, corner_img.clone(), 0, 0, horizontal_sz, FRAC_PI_2);
    add_corner_extra_bounding_box(&mut cmds, 0, 0, vertical_sz);

    // Bottom-left
    add_tile(&mut cmds, corner_img.clone(), map_size.x - 1, 0, horizontal_sz, PI);
    add_corner_extra_bounding_box(&mut cmds, map_size.x - 1, 0, vertical_sz);

    // Top-left
    add_tile(
        &mut cmds,
        corner_img.clone(),
        map_size.x - 1,
        map_size.y - 1,
        horizontal_sz,
        -FRAC_PI_2,
    );
    add_corner_extra_bounding_box(&mut cmds, map_size.x - 1, map_size.y - 1, vertical_sz);

    // Top-right
    add_tile(&mut cmds, corner_img.clone(), 0, map_size.y - 1, horizontal_sz, 0.);
    add_corner_extra_bounding_box(&mut cmds, 0, map_size.y - 1, vertical_sz);
}
