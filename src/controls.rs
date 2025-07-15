#[cfg(feature = "sqlite")]
use crate::prelude::*;
use bevy::prelude::*;
use std::iter::IntoIterator;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_controls);

        #[cfg(feature = "sqlite")]
        app.add_systems(
            Update,
            controls_sync.run_if(resource_changed::<Controls>.and(not(resource_added::<Controls>))),
        );
    }
}

fn setup_controls(mut commands: Commands, #[cfg(feature = "sqlite")] database: Res<Database>) {
    #[cfg(feature = "sqlite")]
    commands.insert_resource(Controls::from_database(&database));
    #[cfg(not(feature = "sqlite"))]
    commands.insert_resource(Controls::default());
}

const KEYBINDS_LEN: usize = 2;
pub type Keybind = [Option<KeyCode>; KEYBINDS_LEN];

pub fn keybind_to_string(code: Option<KeyCode>) -> String {
    match code {
        Some(code) => ron::to_string(&code).unwrap().into(),
        Option::None => "None".into(),
    }
}

/// The list of controls for each input
/// TODO: Implement controller inputs maybe
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct Controls {
    pub move_up: Keybind,
    pub move_down: Keybind,
    pub move_left: Keybind,
    pub move_right: Keybind,
    pub zoom_in: Keybind,
    pub zoom_out: Keybind,
    pub pause: Keybind,
    pub select: Keybind,
}

impl Controls {
    pub fn get_control(&self, control: Control, entry: usize) -> Option<KeyCode> {
        assert!(entry < KEYBINDS_LEN);

        (match control {
            Control::MoveUp => self.move_up,
            Control::MoveDown => self.move_down,
            Control::MoveLeft => self.move_left,
            Control::MoveRight => self.move_right,
            Control::ZoomIn => self.zoom_in,
            Control::ZoomOut => self.zoom_out,
            Control::Pause => self.pause,
            Control::Select => self.select,
        })[entry]
    }
    pub fn set_control(&mut self, control: Control, entry: usize, bind: Option<KeyCode>) {
        assert!(entry < KEYBINDS_LEN);

        (match control {
            Control::MoveUp => &mut self.move_up,
            Control::MoveDown => &mut self.move_down,
            Control::MoveLeft => &mut self.move_left,
            Control::MoveRight => &mut self.move_right,
            Control::ZoomIn => &mut self.zoom_in,
            Control::ZoomOut => &mut self.zoom_out,
            Control::Pause => &mut self.pause,
            Control::Select => &mut self.select,
        })[entry] = bind;
    }

    pub fn reset_control(&mut self, control: Control) {
        match control {
            Control::MoveUp => self.move_up = DEFAULT_UP_CONTROLS,
            Control::MoveDown => self.move_down = DEFAULT_DOWN_CONTROLS,
            Control::MoveLeft => self.move_left = DEFAULT_LEFT_CONTROLS,
            Control::MoveRight => self.move_right = DEFAULT_RIGHT_CONTROLS,
            Control::ZoomIn => self.zoom_in = DEFAULT_ZOOM_IN_CONTROLS,
            Control::ZoomOut => self.zoom_out = DEFAULT_ZOOM_OUT_CONTROLS,
            Control::Pause => self.pause = DEFAULT_PAUSE_CONTROLS,
            Control::Select => self.select = DEFAULT_SELECT_CONTROLS,
        }
    }

    pub fn reset_control_part(&mut self, control: Control, i: usize) {
        assert!(i < KEYBINDS_LEN);

        match control {
            Control::MoveUp => self.move_up[i] = DEFAULT_UP_CONTROLS[i],
            Control::MoveDown => self.move_down[i] = DEFAULT_DOWN_CONTROLS[i],
            Control::MoveLeft => self.move_left[i] = DEFAULT_LEFT_CONTROLS[i],
            Control::MoveRight => self.move_right[i] = DEFAULT_RIGHT_CONTROLS[i],
            Control::ZoomIn => self.zoom_in[i] = DEFAULT_ZOOM_IN_CONTROLS[i],
            Control::ZoomOut => self.zoom_out[i] = DEFAULT_ZOOM_OUT_CONTROLS[i],
            Control::Pause => self.pause[i] = DEFAULT_PAUSE_CONTROLS[i],
            Control::Select => self.select[i] = DEFAULT_SELECT_CONTROLS[i],
        }
    }

    pub fn reset_controls(&mut self) {
        *self = default();
    }
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            move_up: DEFAULT_UP_CONTROLS,
            move_down: DEFAULT_DOWN_CONTROLS,
            move_left: DEFAULT_LEFT_CONTROLS,
            move_right: DEFAULT_RIGHT_CONTROLS,
            zoom_in: DEFAULT_ZOOM_IN_CONTROLS,
            zoom_out: DEFAULT_ZOOM_OUT_CONTROLS,
            pause: DEFAULT_PAUSE_CONTROLS,
            select: DEFAULT_SELECT_CONTROLS,
        }
    }
}

#[cfg(feature = "sqlite")]
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
            select: query_keybind_or_set(database, "select", DEFAULT_SELECT_CONTROLS),
        }
    }
}

#[cfg(feature = "sqlite")]
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
        set_keybind(database, "select", self.select)?;

        Ok(())
    }
}

impl IntoIterator for Controls {
    type Item = (Control, Keybind);
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            controls: self,
            current: Some(default()),
        }
    }
}

#[derive(Default)]
pub struct IntoIter {
    controls: Controls,
    current: Option<Control>,
}

impl Iterator for IntoIter {
    type Item = (Control, Keybind);

    fn next(&mut self) -> Option<Self::Item> {
        self.current.and_then(|control| {
            let res = match control {
                Control::MoveUp => (Control::MoveUp, self.controls.move_up),
                Control::MoveDown => (Control::MoveDown, self.controls.move_down),
                Control::MoveLeft => (Control::MoveLeft, self.controls.move_left),
                Control::MoveRight => (Control::MoveRight, self.controls.move_right),
                Control::ZoomIn => (Control::ZoomIn, self.controls.zoom_in),
                Control::ZoomOut => (Control::ZoomOut, self.controls.zoom_out),
                Control::Pause => (Control::Pause, self.controls.pause),
                Control::Select => (Control::Select, self.controls.select),
            };

            self.current = control.next();

            Some(res)
        })
    }
}

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum Control {
    #[default]
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ZoomIn,
    ZoomOut,
    Pause,
    Select,
}

impl Control {
    pub fn next(self) -> Option<Self> {
        match self {
            Control::MoveUp => Some(Control::MoveDown),
            Control::MoveDown => Some(Control::MoveLeft),
            Control::MoveLeft => Some(Control::MoveRight),
            Control::MoveRight => Some(Control::ZoomIn),
            Control::ZoomIn => Some(Control::ZoomOut),
            Control::ZoomOut => Some(Control::Pause),
            Control::Pause => Some(Control::Select),
            Control::Select => None,
        }
    }

    pub fn as_string(self) -> &'static str {
        match self {
            Control::MoveUp => "Move Up",
            Control::MoveDown => "Move Down",
            Control::MoveLeft => "Move Left",
            Control::MoveRight => "Move Right",
            Control::ZoomIn => "Zoom In",
            Control::ZoomOut => "Zoom Out",
            Control::Pause => "Pause",
            Control::Select => "Select",
        }
    }
}

use std::fmt::{Display, Formatter};
impl Display for Control {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_string())
    }
}

const DEFAULT_UP_CONTROLS: Keybind = [Some(KeyCode::ArrowUp), Some(KeyCode::KeyW)];
const DEFAULT_DOWN_CONTROLS: Keybind = [Some(KeyCode::ArrowDown), Some(KeyCode::KeyS)];
const DEFAULT_LEFT_CONTROLS: Keybind = [Some(KeyCode::ArrowLeft), Some(KeyCode::KeyA)];
const DEFAULT_RIGHT_CONTROLS: Keybind = [Some(KeyCode::ArrowRight), Some(KeyCode::KeyD)];
const DEFAULT_ZOOM_IN_CONTROLS: Keybind = [Some(KeyCode::Comma), None];
const DEFAULT_ZOOM_OUT_CONTROLS: Keybind = [Some(KeyCode::Period), None];
const DEFAULT_PAUSE_CONTROLS: Keybind = [Some(KeyCode::Escape), Some(KeyCode::Pause)];
// TODO: Change this to mouse button left.
const DEFAULT_SELECT_CONTROLS: Keybind = [Some(KeyCode::KeyA), None];

#[cfg(feature = "sqlite")]
fn query_keybind_or_set(database: &Database, keybind: &str, default: Keybind) -> Keybind {
    query_keybind_or_set_fallible(database, keybind, default)
        .inspect_err(|err| {
            warn!("Failed to get keybind: '{keybind}' from sqlite with error: {err}")
        })
        .unwrap_or(default)
}

#[cfg(feature = "sqlite")]
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

#[cfg(feature = "sqlite")]
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

#[cfg(feature = "sqlite")]
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

#[cfg(feature = "sqlite")]
fn controls_sync(database: Res<Database>, controls: Res<Controls>) {
    match controls.to_database(&database) {
        Ok(()) => {}
        Err(err) => {
            warn!("Failed to sync controls to database with: {err}");
        }
    };
}
