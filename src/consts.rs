use bevy_ecs_tilemap::prelude::HexCoordSystem;
use bevy_ecs_tilemap::prelude::TilemapTileSize;
use std::ops::Range;

pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 52.0 };
pub const TILE_ASSET_LOAD_PATH: &'static str = "embedded://assets/sprites/basic_sheet.png";
pub const FLOOR_TILE_VARIENTS: Range<u32> = 0..6;
pub const SKY_TILE_VARIENTS: Range<u32> = 6..14;
pub const HEX_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;
