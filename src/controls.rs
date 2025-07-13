use crate::prelude::*;
use bevy::prelude::*;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_controls).add_systems(
            Update,
            controls_sync.run_if(resource_changed::<Controls>.and(not(resource_added::<Controls>))),
        );
    }
}

fn setup_controls(mut commands: Commands, database: Res<Database>) {
    commands.insert_resource(Controls::from_database(&database));
}

type Keybind = [Option<KeyCode>; 2];

/// The list of controls for each input
/// TODO: Implement controller inputs maybe
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Controls {
    pub move_up: Keybind,
    pub move_down: Keybind,
    pub move_left: Keybind,
    pub move_right: Keybind,
    pub zoom_in: Keybind,
    pub zoom_out: Keybind,
    pub pause: Keybind,
}

// TODO: Do this in a single transaction maybe? (don't know if it matters)
impl FromDatabase for Controls {
    fn from_database(database: &Database) -> Self {
        Self {
            move_up: query_keybind_or_set(database, "move_up", DEFAULT_UP_CONTROLS),
            move_down: query_keybind_or_set(database, "move_down", DEFAULT_DOWN_CONTROLS),
            move_left: query_keybind_or_set(database, "move_left", DEFAULT_LEFT_CONTROLS),
            move_right: query_keybind_or_set(database, "move_right", DEFAULT_RIGHT_CONTROLS),
            zoom_in: query_keybind_or_set(database, "zoom_in", DEFAULT_ZOOM_IN_CONTROLS),
            zoom_out: query_keybind_or_set(database, "zoom_out", DEFAULT_ZOOM_OUT_CONTROLS),
            pause: query_keybind_or_set(database, "pause", DEFAULT_PAUSE_CONTROLS),
        }
    }
}

// TODO: Do this in a single transaction maybe? (don't know if it matters)
impl ToDatabase for Controls {
    type Error = sqlite::Error;
    fn to_database(&self, database: &Database) -> Result<(), Self::Error> {
        set_keybind(database, "move_up", self.move_up)?;
        set_keybind(database, "move_down", self.move_down)?;
        set_keybind(database, "move_left", self.move_left)?;
        set_keybind(database, "move_right", self.move_right)?;
        set_keybind(database, "zoom_in", self.zoom_in)?;
        set_keybind(database, "zoom_out", self.zoom_out)?;
        set_keybind(database, "pause", self.pause)?;

        Ok(())
    }
}

const DEFAULT_UP_CONTROLS: Keybind = [Some(KeyCode::ArrowUp), Some(KeyCode::KeyW)];
const DEFAULT_DOWN_CONTROLS: Keybind = [Some(KeyCode::ArrowDown), Some(KeyCode::KeyS)];
const DEFAULT_LEFT_CONTROLS: Keybind = [Some(KeyCode::ArrowLeft), Some(KeyCode::KeyA)];
const DEFAULT_RIGHT_CONTROLS: Keybind = [Some(KeyCode::ArrowRight), Some(KeyCode::KeyD)];
const DEFAULT_ZOOM_IN_CONTROLS: Keybind = [Some(KeyCode::Comma), None];
const DEFAULT_ZOOM_OUT_CONTROLS: Keybind = [Some(KeyCode::Period), None];
const DEFAULT_PAUSE_CONTROLS: Keybind = [Some(KeyCode::Escape), Some(KeyCode::Pause)];

fn query_keybind_or_set(database: &Database, keybind: &str, default: Keybind) -> Keybind {
    query_keybind_or_set_fallible(database, keybind, default)
        .inspect_err(|err| {
            warn!("Failed to get keybind: '{keybind}' from sqlite with error: {err}")
        })
        .unwrap_or(default)
}

fn query_keybind_or_set_fallible(
    database: &Database,
    keybind: &str,
    default: Keybind,
) -> Result<Keybind, sqlite::Error> {
    Ok(match query_keybind(database, keybind)? {
        Some(kb) => kb,
        Option::None => {
            warn!(
                "Keybind {keybind} not found in database! (this is expected first boot) Inserting now..."
            );
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

    let values = [
        value[0].as_ref().and_then(|v| ron::to_string(v).ok()),
        value[1].as_ref().and_then(|v| ron::to_string(v).ok()),
    ];

    let mut statement = database.connection.prepare(query)?;
    statement.bind((":keybind", keybind))?;
    statement.bind_iter([
        (":key1", values[0].as_deref()),
        (":key2", values[1].as_deref()),
    ])?;

    assert!(matches!(statement.next()?, sqlite::State::Done));

    Ok(())
}

fn controls_sync(_commands: Commands, database: Res<Database>, controls: Res<Controls>) {
    match controls.to_database(&database) {
        Ok(()) => {}
        Err(err) => {
            warn!("Failed to sync controls to database with: {err}");
        }
    };
}
