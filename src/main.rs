mod consts;
use consts::*;

mod camera;
use camera::CameraPlugin;

mod sky;
use sky::SkyPlugin;

#[cfg(feature = "debug")]
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    text::FontSmoothing,
};

use bevy::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, helpers::hex_grid::axial::AxialPos, prelude::*};

use rand::SeedableRng;
use wyrand::WyRand;

pub type RandomSource = WyRand;
//#[derive(Resource)]
//pub struct GlobalRandom(RandomSource);

#[macro_export]
macro_rules! embed_asset {
    ($app: ident, $path: expr) => {{
        let embedded = $app
            .world_mut()
            .resource_mut::<::bevy::asset::io::embedded::EmbeddedAssetRegistry>();

        embedded.insert_asset(
            concat!(env!("CARGO_MANIFEST_DIR"), "/", $path).into(),
            ::std::path::Path::new($path),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)),
        );
    }};
}

fn main() {
    let mut rand = WyRand::from_os_rng();
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RaMmYen Game".into(),
                    ..default()
                }),
                ..default()
            }),
    ); // fallback to nearest sampling

    embed_asset!(app, "assets/sprites/basic_sheet.png");

    #[cfg(feature = "debug")]
    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextFont {
                font_size: 42.0,
                font: default(),
                font_smoothing: FontSmoothing::default(),
                ..default()
            },
            text_color: OVERLAY_COLOR,
            refresh_interval: core::time::Duration::from_millis(100),
            enabled: true,
        },
    });

    // foreign plugins
    app.add_plugins(TilemapPlugin);

    // Local Plugins
    app.add_plugins(SkyPlugin {
        rng: RandomSource::from_rng(&mut rand),
    })
    .add_plugins(CameraPlugin)
    //.insert_resource::<GlobalRandom>(GlobalRandom(rand))
    .add_systems(Startup, spawn_floors)
    .run();
}

const AXIAL_DIRECTIONS: [AxialPos; 7] = [
    AxialPos::new(0, 0),
    AxialPos::new(1, 0),
    AxialPos::new(0, 1),
    AxialPos::new(0, -1),
    AxialPos::new(-1, 0),
    AxialPos::new(1, -1),
    AxialPos::new(-1, 1),
];

fn spawn_floors(mut commands: Commands, asset_server: Res<AssetServer>) {
    let floor_texture = asset_server.load("embedded://assets/sprites/basic_sheet.png");

    AXIAL_DIRECTIONS
        .iter()
        .map(|p| AxialPos::new(p.q * 4, p.r * 4))
        .for_each(|trans| {
            spawn_section(&mut commands, trans, floor_texture.clone());
        });
}

fn spawn_section(commands: &mut Commands, origin: AxialPos, texture: Handle<Image>) {
    let map_size = TilemapSize { x: 1, y: 2 };

    let center =
        (origin - AxialPos { q: 1, r: 0 }).center_in_world_row(&FLOOR_TILE_SIZE.as_vec2().into());

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);
    let hex_coord_system = HexCoordSystem::Row;
    let map_type = TilemapType::Hexagon(hex_coord_system);

    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&TilePos { x: 1, y: 1 }, hex_coord_system),
        1,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(hex_coord_system));

    commands.entity(tilemap_id.0).with_children(|parent| {
        for tile_pos in tile_positions {
            let tile_entity = parent
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.checked_set(&tile_pos, tile_entity)
        }
    });

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: FLOOR_TILE_SIZE.as_vec2().into(),
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture),
        tile_size: FLOOR_TILE_SIZE.as_vec2().into(),
        map_type,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_translation(center.extend(0.)),
        frustum_culling: FrustumCulling(true),
        render_settings: TilemapRenderSettings {
            y_sort: true,
            ..Default::default()
        },
        ..Default::default()
    });
}
