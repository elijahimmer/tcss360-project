mod coords;
use coords::*;

mod sky;
use sky::SkyPlugin;

use bevy::{
    asset::LoadedFolder,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};
use bevy_ecs_tilemap::{helpers::hex_grid::axial::AxialPos, prelude::*, FrustumCulling};
use bevy_pixcam::{PixelCameraPlugin, PixelViewport, PixelZoom};
use bevy_prng::WyRand;
use bevy_rand::prelude::{EntropyPlugin, Entropy};
use bevy_rand::prelude::GlobalEntropy;

const OVERLAY_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);

pub type RandomSource = Entropy<WyRand>;
pub type GlobalRandom<'a> = GlobalEntropy<'a, WyRand>;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RaMmYen Game".into(),
                        ..default()
                    }),
                    ..default()
                }),
        ) // fallback to nearest sampling
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    // Here we define size of our overlay
                    font_size: 42.0,
                    // If we want, we can use a custom font
                    font: default(),
                    // We could also disable font smoothing,
                    font_smoothing: FontSmoothing::default(),
                    ..default()
                },
                // We can also change color of the overlay
                text_color: OVERLAY_COLOR,
                // We can also set the refresh interval for the FPS counter
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
            },
        })
        // foreign plugins
        .add_plugins(PixelCameraPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        // Local Plugins
        .add_plugins(SkyPlugin)
        .init_resource::<SpriteFolder>()
        .init_state::<AppState>()
        .add_systems(Update, check_textures.run_if(in_state(AppState::Loading)))
        .add_systems(OnEnter(AppState::Playing), (setup_camera, spawn_floors))
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Playing,
}

/// The asset folder holding all the sprite sheets.
#[derive(Deref, Resource)]
struct SpriteFolder(Handle<LoadedFolder>);
impl FromWorld for SpriteFolder {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load_folder("sprites"))
    }
}

/// Stalls the game until all of the textures have been loaded.
///
/// Is this necessary?
fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    sprite_folder: Res<SpriteFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&sprite_folder.0) {
            next_state.set(AppState::Playing);
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, PixelZoom::Fixed(2), PixelViewport));
}

fn spawn_floors(mut commands: Commands, asset_server: Res<AssetServer>) {
    let floor_texture = asset_server.load("sprites/basic_sheet.png");

    [
        AxialPos::new(0, 0),
        AxialPos::new(4, 0),
        AxialPos::new(0, 4),
        AxialPos::new(0, -4),
        AxialPos::new(-4, 0),
        AxialPos::new(4, -4),
        AxialPos::new(-4, 4),
    ]
    .iter()
    .for_each(|trans| {
        spawn_section(&mut commands, *trans, floor_texture.clone());
    });
}

fn spawn_section(commands: &mut Commands, origin: AxialPos, texture: Handle<Image>) {
    let map_size = TilemapSize { x: 1, y: 2 };
    let center = (origin - AxialPos { q: 1, r: 0 }).center_in_world_row(&HEX_SIZE.as_vec2().into());

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
        grid_size: HEX_SIZE.as_vec2().into(),
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture),
        tile_size: HEX_SIZE.as_vec2().into(),
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
