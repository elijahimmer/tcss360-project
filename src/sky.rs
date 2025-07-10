//! The infinite sky implementation
use crate::consts::*;
use crate::{RandomSource, embed_asset};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;

const SKY_MAP_SIZE: TilemapSize = TilemapSize { x: 40, y: 24 };
const SKY_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48., y: 52. };
const SKY_TILE_SIZE_LOOP_THRESHOLD: Vec2 = Vec2 {
    x: SKY_TILE_SIZE.x,
    y: SKY_TILE_SIZE.y * 1.5,
};
const SKY_TILE_LAYER: f32 = -1.;
const SKY_TILE_VARIENT_COUNT: u32 = 8;
const SKY_TILE_ASSET_LOAD_PATH: &'static str = "embedded://assets/sprites/sky_sheet.png";

///
pub struct SkyPlugin {
    /// The PRRNG for the sky to use.
    pub rng: RandomSource,
}

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/sky_sheet.png");

        app.init_resource::<SkyMovement>()
            .insert_resource(SkyRand(self.rng.clone()))
            .add_systems(Startup, spawn_sky)
            .add_systems(Update, sky_movement);
    }
}

/// Indicates a tile is a sky tile
#[derive(Component)]
struct SkyTile;

#[derive(Component)]
struct SkyTileMap;

#[derive(Resource)]
struct SkyRand(pub RandomSource);

#[derive(Resource)]
struct SkyMovement {
    /// The speed of movement in tiles per second, in axial coordinates.
    pub speed: Vec2,
}

impl Default for SkyMovement {
    fn default() -> Self {
        Self {
            speed: Vec2::new(5., 2.),
        }
    }
}

/// Spawns the sky fitting the screen (to an extent).
fn spawn_sky(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<SkyRand>) {
    let texture_handle: Handle<Image> = asset_server.load(SKY_TILE_ASSET_LOAD_PATH);

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(SKY_MAP_SIZE);

    for x in 0..SKY_MAP_SIZE.x {
        for y in 0..SKY_MAP_SIZE.y {
            let tile_pos = TilePos { x, y };
            commands.entity(tilemap_entity).with_children(|parent| {
                let tile_entity = parent
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                            texture_index: TileTextureIndex(
                                rng.0.random_range(0..SKY_TILE_VARIENT_COUNT),
                            ),
                            ..Default::default()
                        },
                        SkyTile,
                    ))
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            });
        }
    }

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size: SKY_TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: SKY_MAP_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size: SKY_TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., SKY_TILE_LAYER),
            ..Default::default()
        },
        SkyTileMap,
    ));
}

/// Moves the sky with an illusion that it is indefinite.
///
/// This system
///
fn sky_movement(
    time: Res<Time>,
    sky_movement: ResMut<SkyMovement>,
    mut rng: ResMut<SkyRand>,
    mut tilemap: Single<(&TileStorage, &TilemapSize, &mut Transform), With<SkyTileMap>>,
    mut tile_query: Query<&mut TileTextureIndex, With<SkyTile>>,
) {
    let map_size: IVec2 = IVec2::new(tilemap.1.x as i32, tilemap.1.y as i32);
    let tile_storage = tilemap.0;

    let old_translation = tilemap.2.translation.xy();
    let mut new_translation =
        AXIAL_TRANSLATION_MATRIX * sky_movement.speed * time.delta_secs() + old_translation;

    let tile_diff = (new_translation / SKY_TILE_SIZE_LOOP_THRESHOLD)
        .trunc()
        .as_ivec2();

    // only translate by the sky by the amount that was less than a whole tile.
    new_translation -= tile_diff.as_vec2() * SKY_TILE_SIZE_LOOP_THRESHOLD;

    tilemap.2.translation = new_translation.extend(tilemap.2.translation.z);

    if tile_diff == IVec2::ZERO {
        return;
    }

    let flip_x = tile_diff.x > 0;
    let flip_y = tile_diff.y > 0;

    for y in 0..map_size.y {
        let y = flip_y.then_some(map_size.y - y - 1).unwrap_or(y);
        for x in 0..map_size.x {
            let x = flip_x.then_some(map_size.x - x - 1).unwrap_or(x);

            let old_pos = IVec2 { x, y };

            // for the hexagons to align with where you started, they have
            // to move 1.5 hexes up or 1 hex to the right.
            // This does the 1.5 hexes up adjustment to turn the
            // hex distance into square distance used by the position.
            let adjusted_diff =
                (Mat2::from_cols_array(&[1., 0., -1., 2.]) * tile_diff.as_vec2()).as_ivec2();

            let replace_pos = old_pos + adjusted_diff;
            let new_pos = old_pos - adjusted_diff;

            let Some(curr_tile_entity) = tile_storage.get(&old_pos.as_uvec2().into()) else {
                warn!("Failed to find sky tile entity at position ({x}, {y})");
                continue;
            };

            if replace_pos.cmpge(IVec2::ZERO).all() && replace_pos.cmplt(map_size).all() {
                // move the texture along the `tile_diff` vector

                let Some(new_tile_entity) = tile_storage.get(&replace_pos.as_uvec2().into()) else {
                    warn!("Failed to find new tile at pos {replace_pos}");
                    continue;
                };

                let curr_tile_texture = match tile_query.get(curr_tile_entity).and_then(|t| Ok(*t))
                {
                    Ok(curr_tile_texture) => curr_tile_texture,
                    Err(err) => {
                        warn!("Failed to find base sky tile at {old_pos} with {err}");
                        continue;
                    }
                };

                match tile_query.get_mut(new_tile_entity) {
                    Ok(mut new_tile_texture) => *new_tile_texture = curr_tile_texture,
                    Err(err) => {
                        warn!("Failed to find to be replaced sky tile at {replace_pos} with {err}")
                    }
                }
            }

            if new_pos.cmplt(IVec2::ZERO).any() || new_pos.cmpge(map_size).any() {
                match tile_query.get_mut(curr_tile_entity) {
                    Ok(mut curr_tile_texture) => {
                        let tile_idx = rng.0.random_range(0..SKY_TILE_VARIENT_COUNT);
                        *curr_tile_texture = TileTextureIndex(tile_idx);
                    }
                    Err(err) => warn!("Failed to get current tile at {new_pos} with {err}"),
                };
            }
        }
    }
}
