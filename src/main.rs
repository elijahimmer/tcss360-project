mod coords;
use coords::*;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_systems(Startup, setup)
        .run();
}

const WIDTH_OFFSET: f32 = -HEX_WIDTH * 5.;
const HEIGHT_OFFSET: f32 = 0.;
const HEX_SIZE: f32 = HEX_HEIGHT / 2.;
const HEX_WIDTH: f32 = 23. * 2.;
const HEX_HEIGHT: f32 = 29. * 2.;
const NUM_WIDE: usize = 8;
const NUM_TALL: usize = 8;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let base_hex = asset_server.load("BaseHexagon.png");

    for i in 0..(NUM_WIDE * NUM_TALL) {
        let final_sprite = Sprite {
            image: base_hex.clone(),
            image_mode: SpriteImageMode::Scale(ScalingMode::FitCenter),
            color: Color::hsl(360. * i as f32 / 256. as f32, 0.95, 0.7),
            custom_size: Some(Vec2{
                x: HEX_WIDTH,
                y: HEX_HEIGHT,

            }),
            ..default()
        };

        let col = i % NUM_WIDE;
        let row = i / NUM_WIDE;
        let row_offset_rel: f32 = (row & 1 == 0).then_some(0.5).unwrap_or(0.);

        // Distribute colors evenly across the rainbow.
        commands.spawn((
            final_sprite,
            Transform::from_xyz(
                (col as f32 + row_offset) * HEX_WIDTH
                    + WIDTH_OFFSET,
                (i / NUM_WIDE) as f32 * HEX_HEIGHT * 3. / 4. + HEIGHT_OFFSET,
                0.,
            ),
        ));
    }
}
