use bevy::prelude::ops::powf;
use bevy::prelude::*;

/// The plugin to enable the camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraMovementSettings>()
            .register_type::<CameraControls>()
            .register_type::<MainCamera>()
            .init_resource::<CameraControls>()
            .init_resource::<CameraMovementSettings>()
            .add_systems(Startup, camera_setup)
            .add_systems(Update, (camera_movement, camera_zoom));
    }
}

/// The camera movement settings for the [`MainCamera`]
#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct CameraMovementSettings {
    /// The movement speed of the camera in in-game pixels per second
    move_speed: f32,

    /// The zoom speed of the camera defined as `zoom *= speed ^ (delta seconds)`
    /// This uses the inverse of the speed when zooming in.
    zoom_speed: f32,

    /// The allowed area the camera can be in, where
    /// each column is the range of movement for a direction.
    /// The first column is the `x` direction, and the second is `y`
    move_area: Mat2,

    /// The bounds of the zoom, `x` being the lower bound and `y` being the upper bound.
    zoom_limit: Vec2,
}

impl Default for CameraMovementSettings {
    fn default() -> Self {
        Self {
            move_speed: 300.0,
            zoom_speed: 4.0,
            move_area: Mat2::from_cols_array_2d(&[[-200.0, 200.0]; 2]),
            zoom_limit: Vec2::new(0.25, 1.0),
        }
    }
}

/// The list of controls for each input
/// TODO: Implement controller inputs maybe
#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct CameraControls {
    up: Vec<KeyCode>,
    down: Vec<KeyCode>,
    left: Vec<KeyCode>,
    right: Vec<KeyCode>,
    zoom_in: Vec<KeyCode>,
    zoom_out: Vec<KeyCode>,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            up: vec![KeyCode::ArrowUp, KeyCode::KeyW],
            down: vec![KeyCode::ArrowDown, KeyCode::KeyS],
            left: vec![KeyCode::ArrowLeft, KeyCode::KeyA],
            right: vec![KeyCode::ArrowRight, KeyCode::KeyD],
            zoom_in: vec![KeyCode::Comma],
            zoom_out: vec![KeyCode::Period],
        }
    }
}

/// The marker component to signify a camera is the main rendering camera
#[derive(Component, Reflect)]
#[reflect(Component)]
struct MainCamera;

/// Sets up the main camera and it's settings
fn camera_setup(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
        Transform::IDENTITY,
    ));
}

/// Returns a floating point number in the range [0, 1] representing if any of the keys are
/// pressed
fn sum_inputs(input: &Res<ButtonInput<KeyCode>>, keys: &[KeyCode]) -> f32 {
    keys.iter()
        .map(|key| input.pressed(*key) as u8)
        .sum::<u8>()
        .clamp(0, 1) as f32
}

/// Controls the camera's translational movement based
/// on user input.
fn camera_movement(
    mut transform: Single<&mut Transform, With<MainCamera>>,
    input: Res<ButtonInput<KeyCode>>,
    settings: Res<CameraMovementSettings>,
    controls: Res<CameraControls>,
    time: Res<Time<Fixed>>,
) {
    let movement = Vec2::Y * sum_inputs(&input, &controls.up)
        + Vec2::NEG_Y * sum_inputs(&input, &controls.down)
        + Vec2::NEG_X * sum_inputs(&input, &controls.left)
        + Vec2::X * sum_inputs(&input, &controls.right);

    let movement = movement * time.delta_secs() * settings.move_speed;

    transform.translation = (transform.translation.xy() + movement)
        .clamp(settings.move_area.row(0), settings.move_area.row(1))
        .extend(transform.translation.z);
}

/// Controls the camera's zoom based on user input.
fn camera_zoom(
    mut projection: Single<&mut Projection, With<MainCamera>>,
    input: Res<ButtonInput<KeyCode>>,
    settings: Res<CameraMovementSettings>,
    controls: Res<CameraControls>,
    time: Res<Time<Fixed>>,
) {
    let Projection::Orthographic(ref mut projection2d) = **projection else {
        unreachable!("Only Orthographic Projection is supported!");
    };

    let scale = projection2d.scale
        * powf(
            powf(settings.zoom_speed, time.delta_secs()),
            sum_inputs(&input, &controls.zoom_in),
        )
        * powf(
            powf(1.0 / settings.zoom_speed, time.delta_secs()),
            sum_inputs(&input, &controls.zoom_out),
        );

    projection2d.scale = scale.clamp(settings.zoom_limit.x, settings.zoom_limit.y);
}
