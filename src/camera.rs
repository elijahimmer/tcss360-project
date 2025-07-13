use crate::prelude::*;
use bevy::prelude::ops::powf;
use bevy::prelude::*;

/// The plugin to enable the camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraMovementSettings>()
            .register_type::<CameraControls>()
            .register_type::<MainCamera>()
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

type Keybind = [Option<KeyCode>; 2];

/// The list of controls for each input
/// TODO: Implement controller inputs maybe
#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct CameraControls {
    up: Keybind,
    down: Keybind,
    left: Keybind,
    right: Keybind,
    zoom_in: Keybind,
    zoom_out: Keybind,
}

impl FromDatabase for CameraControls {
    fn from_database(database: &Database) -> Self {
        Self {
            up: query_keybind_or_set(database, "move_up", DEFAULT_UP_CONTROLS),
            down: query_keybind_or_set(database, "move_down", DEFAULT_DOWN_CONTROLS),
            left: query_keybind_or_set(database, "move_left", DEFAULT_LEFT_CONTROLS),
            right: query_keybind_or_set(database, "move_right", DEFAULT_RIGHT_CONTROLS),
            zoom_in: query_keybind_or_set(database, "zoom_in", DEFAULT_ZOOM_IN_CONTROLS),
            zoom_out: query_keybind_or_set(database, "zoom_out", DEFAULT_ZOOM_OUT_CONTROLS),
        }
    }
}

const DEFAULT_UP_CONTROLS: Keybind = [Some(KeyCode::ArrowUp), Some(KeyCode::KeyW)];
const DEFAULT_DOWN_CONTROLS: Keybind = [Some(KeyCode::ArrowDown), Some(KeyCode::KeyS)];
const DEFAULT_LEFT_CONTROLS: Keybind = [Some(KeyCode::ArrowLeft), Some(KeyCode::KeyA)];
const DEFAULT_RIGHT_CONTROLS: Keybind = [Some(KeyCode::ArrowRight), Some(KeyCode::KeyD)];
const DEFAULT_ZOOM_IN_CONTROLS: Keybind = [Some(KeyCode::Comma), None];
const DEFAULT_ZOOM_OUT_CONTROLS: Keybind = [Some(KeyCode::Period), None];


fn query_keybind_or_set(database: &Database, keybind: &str, default: Keybind) -> Keybind {
    query_keybind_or_set_fallible(database, keybind, default)
        .inspect_err(|err| warn!("Failed to get keybind: '{keybind}' from sqlite with error: {err}"))
        .unwrap_or(default)
}

fn query_keybind_or_set_fallible(database: &Database, keybind: &str, default: Keybind) -> Result<Keybind, sqlite::Error> {
    Ok(match query_keybind(database, keybind)? {
        Some(kb) => kb,
        None => {
            warn!("Keybind {keybind} not found in database! (this is expected first boot) Inserting now...");
            set_keybind(database, keybind, default)?;

            default
        }
    })
}

fn query_keybind(database: &Database, keybind: &str) -> Result<Option<Keybind>, sqlite::Error> {
    let query = "SELECT key1,key2 FROM Keybinds WHERE keybind = :keybind";

    let mut statement = database.connection.prepare(query)?;
    statement.bind((":keybind", keybind))?;

    if let sqlite::State::Done = statement.next()? {
        return Ok(None);
    }
    assert_eq!(
        statement.column_count(),
        2,
        "There should only be 3 columns in this table"
    );

    // read the value column index.
    let key1 = statement.read::<Option<String>, usize>(0)?;
    let key2 = statement.read::<Option<String>, usize>(1)?;

    assert!(matches!(statement.next()?, sqlite::State::Done));

    Ok(Some([
        key1.and_then(|v| ron::from_str(&v).ok()),
        key2.and_then(|v| ron::from_str(&v).ok()),
    ]))
}

fn set_keybind(database: &Database, keybind: &str, value: Keybind) -> Result<(), sqlite::Error> {
    let query = "INSERT OR REPLACE INTO Keybinds VALUES (:keybind, :key1, :key2)";

    let values =[
        value[0].as_ref().and_then(|v| ron::to_string(v).ok()),
        value[1].as_ref().and_then(|v| ron::to_string(v).ok())
    ];

    let mut statement = database.connection.prepare(query)?;
    statement.bind((":keybind", keybind))?;
    statement.bind_iter([
        (":key1", values[0].as_deref()),
        (":key2", values[1].as_deref())
    ])?;

    assert!(matches!(statement.next()?, sqlite::State::Done));

    Ok(())
}

/// The marker component to signify a camera is the main rendering camera
#[derive(Component, Reflect)]
#[reflect(Component)]
struct MainCamera;

/// Sets up the main camera and it's settings
fn camera_setup(mut commands: Commands, database: Res<Database>) {
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

    commands.insert_resource(CameraControls::from_database(&database));
}

/// Returns a floating point number in the range [0, 1] representing if any of the keys are
/// pressed
fn sum_inputs(input: &Res<ButtonInput<KeyCode>>, keys: &[Option<KeyCode>]) -> f32 {
    keys.into_iter()
        .map(|key| key.is_some_and(|k| input.pressed(k)) as u8)
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
