use bevy::prelude::*;

/// TODO: Replace with `std::f32::consts::SQRT_3` when that is stable.
//pub const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;
pub const SQRT_3_2: f32 = 0.866025403784438646763723170752936183_f32;

/// The full hex size
pub const FLOOR_TILE_SIZE: IVec2 = IVec2 { x: 24, y: 26 };

#[cfg(feature = "debug")]
pub const OVERLAY_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);

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
