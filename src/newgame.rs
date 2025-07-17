use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::prelude::*;
use rand::{Rng, SeedableRng};

pub struct NewGamePlugin;

const ROOM_TILE_VARIENT_COUNT: u32 = 7; 
const ROOM_SIZE: TilemapSize = TilemapSize { x: 50, y: 50 };
const ROOM_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 52.0 };
const BASIC_TILE_ASSET_LOAD_PATH: &'static str = "embedded://assets/sprites/basic_sheet.png";

const ROOM_TILE_LAYER: f32 = -1.;

impl Plugin for NewGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TileRand(RandomSource::from_os_rng()))
            .add_systems(OnEnter(GameState::Game), spawn_room);
    }
}


#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTile;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTileMap;


#[derive(Resource)]
struct TileRand(pub RandomSource);

fn spawn_room(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    mut rng: ResMut<TileRand> 
) {
    let texture_handle: Handle<Image> = asset_server.load(BASIC_TILE_ASSET_LOAD_PATH);
    
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(ROOM_SIZE);

    commands.entity(tilemap_entity).with_children(|parent| {
        for x in 0..(ROOM_SIZE.x) {
            for y in 0..(ROOM_SIZE.y) {
                let tile_pos = TilePos { x, y };
                let id = parent
                    .spawn((
                        RoomTile,
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                            texture_index: TileTextureIndex(
                                rng.0.random_range(0..ROOM_TILE_VARIENT_COUNT),
                            ),
                            ..Default::default()
                        },
                    ))
                    .id();
                tile_storage.set(&tile_pos, id);
            }
        }
    });

    commands.entity(tilemap_entity).insert((
        RoomTileMap,
        TilemapBundle {
            grid_size: ROOM_TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: ROOM_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size: ROOM_TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
            ..Default::default()
        },
    ));
}

/*
fn test_enter() {
    for _i in 0..20 {
        println!("working");
    }
}

*/
