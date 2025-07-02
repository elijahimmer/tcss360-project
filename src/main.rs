mod coords;
use coords::*;

use bevy::{asset::LoadedFolder, prelude::*, sprite::Anchor};
use bevy_ecs_tilemap::prelude::*;
use bevy_pixcam::{PixelCameraPlugin, PixelViewport, PixelZoom};
use rand::{prelude::*, rngs::SmallRng};

const SCREEN_WIDTH: u32 = 480;
const SCREEN_HEIGHT: u32 = 270;

const FLOOR_TILE_ATLAS_WIDTH: u32 = 1;
const FLOOR_TILE_ATLAS_HEIGHT: u32 = 7;
const FLOOR_TILE_PADDING: Option<UVec2> = Some(UVec2 { x: 2, y: 0 });

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "KhImNgu Game".into(),
                        ..default()
                    }),
                    ..default()
                }),
        ) // fallback to nearest sampling
        .add_plugins(PixelCameraPlugin)
        .add_plugins(TilemapPlugin)
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), init_resources)
        .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))
        .add_systems(
            OnEnter(AppState::Finished),
            (setup_camera, spawn_floors, spawn_sky),
        )
        .add_systems(Update, sky_movement.run_if(in_state(AppState::Finished)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

/// Indicates a tile is a sky tile
#[derive(Component)]
struct SkyTile;

#[derive(Component)]
struct SkyTileMap;

#[derive(Resource)]
struct SkyMovement {
    /// The speed of movement in tiles per second, in axial coordinates.
    speed: Vec2,
}

/// The source of randomness used by this example.
#[derive(Resource)]
struct RandomSource(SmallRng);

/// The asset folder holding all the sprite sheets.
#[derive(Resource)]
struct SpriteFolder(Handle<LoadedFolder>);

fn init_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load multiple, individual sprites from a folder
    commands.insert_resource(SpriteFolder(asset_server.load_folder("sprites")));

    commands.insert_resource(SkyMovement {
        speed: Vec2::new(5., 3.),
    });

    commands.insert_resource(RandomSource(SmallRng::from_os_rng()));
}

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    sprite_folder: Res<SpriteFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&sprite_folder.0) {
            next_state.set(AppState::Finished);
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PixelZoom::FitSize {
            width: SCREEN_WIDTH as i32,
            height: SCREEN_HEIGHT as i32,
        },
        PixelViewport,
    ));
}

fn spawn_floors(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Create texture atlases with different padding and sampling
    let floor_texture = asset_server.load("sprites/basic_sheet.png");
    let floor_layout = TextureAtlasLayout::from_grid(
        HEX_SIZE.as_uvec2(),
        FLOOR_TILE_ATLAS_HEIGHT,
        FLOOR_TILE_ATLAS_WIDTH,
        FLOOR_TILE_PADDING,
        None,
    );
    let floor_texture_atlas_layout = texture_atlas_layouts.add(floor_layout);

    spawn_section(
        &mut commands,
        IVec2::ZERO,
        floor_texture.clone(),
        floor_texture_atlas_layout.clone(),
    );

    Direction::ALL
        .iter()
        .map(|dir| dir.as_ivec2() * 4)
        .for_each(|trans| {
            spawn_section(
                &mut commands,
                trans,
                floor_texture.clone(),
                floor_texture_atlas_layout.clone(),
            );
        });
}

fn spawn_section(
    commands: &mut Commands,
    center: IVec2,
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
) {
    let sprite = Sprite {
        image: texture.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: layout.clone(),
            index: 0,
        }),
        anchor: Anchor::Center,
        ..Default::default()
    };
    commands.spawn((
        sprite.clone(),
        Transform::from_translation(center.as_vec2().extend(0.)),
    ));

    Direction::ALL
        .iter()
        .map(|dir| (center + dir.as_ivec2()).as_vec2().extend(0.))
        .for_each(|trans| {
            commands.spawn((sprite.clone(), Transform::from_translation(trans)));
        });
}

fn spawn_sky(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RandomSource>,
) {
    let texture_handle: Handle<Image> = asset_server.load("sprites/sky_sheet.png");

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(SKY_MAP_SIZE);

    for x in 0..SKY_MAP_SIZE.x {
        for y in 0..SKY_MAP_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(rng.0.random_range(0..SKY_TILE_VARIENT_COUNT)),
                        //color: TileColor(Color::srgba(0.0, 0.0, 0.0, 1.0)),
                        ..Default::default()
                    },
                    SkyTile,
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let grid_size = SKY_TILE_SIZE.into();
    let map_type = TilemapType::Hexagon(HexCoordSystem::Row);

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
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

fn sky_movement(
    time: Res<Time<Fixed>>,
    sky_movement: ResMut<SkyMovement>,
    mut rng: ResMut<RandomSource>,
    mut tilemap: Single<(&TileStorage, &TilemapSize, &mut Transform), With<SkyTileMap>>,
    mut tile_query: Query<&mut TileTextureIndex, With<SkyTile>>,
) {
    let map_size = IVec2::new(tilemap.1.x as i32, tilemap.1.y as i32);
    let tile_storage = tilemap.0;

    let old_translation = tilemap.2.translation.xy();
    let mut new_translation =
        AXIAL_TRANSLATION_MATRIX * sky_movement.speed * time.delta_secs() + old_translation;

    let tile_diff = (new_translation / SKY_TILE_SIZE_LOOP_THRESHOLD)
        .trunc()
        .as_ivec2();


    if tile_diff != IVec2::ZERO {
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
                let adjusted_diff = (Mat2::from_cols_array_2d(&[
                    [1., 0.],
                    [-1., 2.]
                ]) * tile_diff.as_vec2()).as_ivec2();
                let replace_pos = old_pos + adjusted_diff;
                let new_pos = old_pos - adjusted_diff;

                let Some(curr_tile_entity) = tile_storage.get(&TilePos {
                    x: x as u32,
                    y: y as u32,
                }) else {
                    continue;
                };

                if replace_pos.cmpge(IVec2::ZERO).all() && replace_pos.cmplt(map_size).all() {
                    // move the texture along the `tile_diff` vector

                    let Some(new_tile_entity) = tile_storage.get(&TilePos {
                        x: replace_pos.x as u32,
                        y: replace_pos.y as u32,
                    }) else {
                        continue;
                    };

                    let Ok(curr_tile_texture) =
                        tile_query.get(curr_tile_entity).and_then(|t| Ok(*t))
                    else {
                        continue;
                    };

                    match tile_query.get_mut(new_tile_entity) {
                        Ok(mut new_tile_texture) => *new_tile_texture = curr_tile_texture,
                        Err(err) => warn!("Failed to find replacing sky tile at {replace_pos} with {err}"),
                    }
                } else {
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

    new_translation -= tile_diff.as_vec2() * SKY_TILE_SIZE_LOOP_THRESHOLD;

    tilemap.2.translation = new_translation.extend(tilemap.2.translation.z);
}

const SKY_MAP_SIZE: TilemapSize = TilemapSize { x: 16, y: 12 };
const SKY_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48., y: 52. };
const SKY_TILE_SIZE_LOOP_THRESHOLD: Vec2 = Vec2 {
    x: SKY_TILE_SIZE.x,
    y: SKY_TILE_SIZE.y * 1.5,
};
const SKY_TILE_LAYER: f32 = -1.;
const SKY_TILE_VARIENT_COUNT: u32 = 8;
