//! In this example we generate four texture atlases (sprite sheets) from a folder containing
//! individual sprites.
//!
//! The texture atlases are generated with different padding and sampling to demonstrate the
//! effect of these settings, and how bleeding issues can be resolved by padding the sprites.
//!
//! Only one padded and one unpadded texture atlas are rendered to the screen.
//! An upscaled sprite from each of the four atlases are rendered to the screen.

mod coords;
use coords::*;

use bevy::{
    asset::LoadedFolder,
    color::palettes::css::GRAY,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::WindowResized,
};

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

/// In-game resolution width.
const RES_WIDTH: u32 = 160;

/// In-game resolution height.
const RES_HEIGHT: u32 = 90;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // fallback to nearest sampling
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), load_textures)
        .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))
        .add_systems(
            OnEnter(AppState::Finished),
            (setup_camera, setup_sprites).chain(),
        )
        .add_systems(Update, (fit_canvas, sprite_movement))
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;

/// The asset folder holding all the sprite sheets.
#[derive(Resource, Default)]
struct SpriteFolder(Handle<LoadedFolder>);

fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load multiple, individual sprites from a folder
    commands.insert_resource(SpriteFolder(asset_server.load_folder("sprites")));
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

fn setup_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Create texture atlases with different padding and sampling
    let texture = asset_server.load("sprites/basic_sheet.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2 { x: 24, y: 29 },
        7,
        1,
        Some(UVec2 { x: 2, y: 0 }),
        None,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
        ),
        Transform::IDENTITY,
        PIXEL_PERFECT_LAYERS,
    ));

    Direction::ALL
        .iter()
        .map(|dir| {
            (
                dir,
                <coords::Direction as Into<Vec2>>::into(*dir).extend(0.),
            )
        })
        .zip(1..)
        .map(|((dir, trans), i)| {
            (
                Sprite::from_atlas_image(
                    texture.clone(),
                    TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: i,
                    },
                ),
                Transform::from_translation(trans * Vec3::new(24., 29., 1.)),
                PIXEL_PERFECT_LAYERS,
                *dir,
            )
        })
        .for_each(|s| {
            commands.spawn(s);
        });
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // This Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // Fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // This camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2d,
        Camera {
            // Render before the "main pass" camera
            order: -1,
            target: RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(GRAY.into()),
            ..default()
        },
        Msaa::Off,
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    // Spawn the canvas
    commands.spawn((Sprite::from_image(image_handle), Canvas, HIGH_RES_LAYERS));

    // The "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2d, Msaa::Off, OuterCamera, HIGH_RES_LAYERS));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        transform.translation +=
            <coords::Direction as Into<Vec2>>::into(*logo).extend(0.) * (50. * time.delta_secs());

        if !(-(RES_WIDTH as f32)..RES_WIDTH as f32).contains(&transform.translation.x) {
            *logo = logo.invert_x();
        }

        if !(-(RES_HEIGHT as f32)..RES_HEIGHT as f32).contains(&transform.translation.y) {
            *logo = logo.invert_y();
        }
    }
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projection: Single<&mut Projection, With<OuterCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}
