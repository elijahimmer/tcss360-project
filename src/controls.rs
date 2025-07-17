//! TODO: Display keybinds as icons/characters and inputs and lists of them.

use crate::embed_asset;
use crate::prelude::*;
use bevy::ecs::relationship::{RelatedSpawnerCommands, Relationship};
use bevy::{input::InputSystem, prelude::*};
use serde::{Deserialize, Serialize};
use std::iter::IntoIterator;

const BUTTON_SPRITE_IMAGE_PATH: &str = "embedded://assets/sprites/buttons.png";

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/buttons.png");

        app.add_systems(Startup, setup_controls)
            .init_resource::<InputState>()
            .init_resource::<ButtonInput<Input>>()
            .add_systems(
                PreUpdate,
                (update_input_state, update_control_state)
                    .chain()
                    .after(InputSystem),
            );

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

pub type InputState = ButtonInput<Control>;

fn update_input_state(
    mut input_state: ResMut<ButtonInput<Input>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepad: Query<&Gamepad>,
) {
    input_state.bypass_change_detection().clear();

    for pressed in keyboard.get_just_pressed() {
        input_state.press(Input::Keyboard(*pressed));
    }

    for released in keyboard.get_just_released() {
        input_state.release(Input::Keyboard(*released));
    }

    for pressed in mouse.get_just_pressed() {
        input_state.press(Input::Mouse(*pressed));
    }

    for released in mouse.get_just_released() {
        input_state.release(Input::Mouse(*released));
    }

    for gamepad in gamepad.iter() {
        for pressed in gamepad.digital().get_just_pressed() {
            input_state.press(Input::Gamepad(*pressed));
        }

        for released in gamepad.digital().get_just_released() {
            input_state.release(Input::Gamepad(*released));
        }
    }
}

fn update_control_state(
    mut control_state: ResMut<InputState>,
    input_state: Res<ButtonInput<Input>>,
    controls: Res<Controls>,
) {
    // Avoid clearing if it's not empty to ensure change detection is not triggered.
    control_state.bypass_change_detection().clear();

    for Keybind(control, keybind) in controls.clone().into_iter() {
        let keybind = keybind.into_iter().filter_map(|k| k);
        // TODO: Remove vec because for something like a bounded array.
        let pressed = input_state.any_pressed(keybind.clone());
        let just_pressed = input_state.any_just_pressed(keybind.clone());
        let just_released = input_state.any_just_released(keybind);

        if just_pressed {
            control_state.press(control);
        }

        if just_released && !pressed {
            control_state.release(control);
        }
    }
}

/// All of the information about an individual keybind
#[derive(Debug, Hash, PartialEq, Eq, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct Keybind(pub Control, pub InputList);

impl Keybind {
    pub fn to_screen<R: Relationship>(
        &self,
        _style: &Style,
        _builder: &mut RelatedSpawnerCommands<'_, R>,
    ) {
        todo!("display multiple");
    }
}

const TEXT_COLOR: Color = Color::srgb_u8(0xe0, 0xde, 0xf4);

pub fn input_to_screen<R: Relationship>(
    style: &Style,
    builder: &mut RelatedSpawnerCommands<'_, R>,
    input: &Option<Input>,
) {
    match input {
        Some(input) => input.to_screen(style, builder),
        None => {
            builder.spawn((
                Text::new("Not Bound"),
                TextFont {
                    font: style.font.clone(),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Label,
                Pickable::IGNORE,
            ));
        }
    }
}

/// The number of keybinds associated with a given control.
/// When changed, the update must be reflected in the database
/// so that we sync all of them correctly.
const INPUT_LIST_LEN: usize = 2;
/// An individual set of inputs for a keybind
pub type InputList = [Option<Input>; INPUT_LIST_LEN];

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Input {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    Gamepad(GamepadButton),
}

impl Input {
    pub fn to_screen<R: Relationship>(
        &self,
        style: &Style,
        builder: &mut RelatedSpawnerCommands<'_, R>,
    ) {
        match self {
            //Self::Keyboard(KeyCode::KeyW) => {
            //    let icon = style.icons;
            //    builder.spawn((
            //        ImageNode::new(icon),
            //        Node {
            //            // This will set the logo to be 200px wide, and auto adjust its height
            //            width: Val::Px(200.0),
            //            ..default()
            //        },
            //    ));
            //}
            _ => {
                builder.spawn((
                    Text::new(ron::to_string(self).unwrap()),
                    style.font(33.0),
                    TextColor(TEXT_COLOR),
                    Label,
                    Pickable::IGNORE,
                ));
            }
        }
    }
}

/// The list of controls for each input
/// TODO: Implement controller inputs maybe
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct Controls {
    pub move_up: InputList,
    pub move_down: InputList,
    pub move_left: InputList,
    pub move_right: InputList,
    pub zoom_in: InputList,
    pub zoom_out: InputList,
    pub pause: InputList,
    pub select: InputList,
}

impl Controls {
    pub fn get_control_mut(&mut self, control: Control) -> &mut InputList {
        match control {
            Control::MoveUp => &mut self.move_up,
            Control::MoveDown => &mut self.move_down,
            Control::MoveLeft => &mut self.move_left,
            Control::MoveRight => &mut self.move_right,
            Control::ZoomIn => &mut self.zoom_in,
            Control::ZoomOut => &mut self.zoom_out,
            Control::Pause => &mut self.pause,
            Control::Select => &mut self.select,
        }
    }

    pub fn get_control(&self, control: Control) -> InputList {
        match control {
            Control::MoveUp => self.move_up,
            Control::MoveDown => self.move_down,
            Control::MoveLeft => self.move_left,
            Control::MoveRight => self.move_right,
            Control::ZoomIn => self.zoom_in,
            Control::ZoomOut => self.zoom_out,
            Control::Pause => self.pause,
            Control::Select => self.select,
        }
    }

    pub fn get_control_part(&self, control: Control, entry: usize) -> Option<Input> {
        assert!(entry < INPUT_LIST_LEN);

        (self.get_control(control))[entry]
    }

    pub fn set_control(&mut self, control: Control, entry: usize, bind: Option<Input>) {
        assert!(entry < INPUT_LIST_LEN);

        self.get_control_mut(control)[entry] = bind;
    }

    pub fn reset_control(&mut self, control: Control) {
        *self.get_control_mut(control) = match control {
            Control::MoveUp => DEFAULT_UP_CONTROLS,
            Control::MoveDown => DEFAULT_DOWN_CONTROLS,
            Control::MoveLeft => DEFAULT_LEFT_CONTROLS,
            Control::MoveRight => DEFAULT_RIGHT_CONTROLS,
            Control::ZoomIn => DEFAULT_ZOOM_IN_CONTROLS,
            Control::ZoomOut => DEFAULT_ZOOM_OUT_CONTROLS,
            Control::Pause => DEFAULT_PAUSE_CONTROLS,
            Control::Select => DEFAULT_SELECT_CONTROLS,
        }
    }

    pub fn reset_control_part(&mut self, control: Control, i: usize) {
        assert!(i < INPUT_LIST_LEN);

        self.get_control_mut(control)[i] = match control {
            Control::MoveUp => DEFAULT_UP_CONTROLS,
            Control::MoveDown => DEFAULT_DOWN_CONTROLS,
            Control::MoveLeft => DEFAULT_LEFT_CONTROLS,
            Control::MoveRight => DEFAULT_RIGHT_CONTROLS,
            Control::ZoomIn => DEFAULT_ZOOM_IN_CONTROLS,
            Control::ZoomOut => DEFAULT_ZOOM_OUT_CONTROLS,
            Control::Pause => DEFAULT_PAUSE_CONTROLS,
            Control::Select => DEFAULT_SELECT_CONTROLS,
        }[i];
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

// TODO: Do this in a single transaction maybe? (don't know if it matters)
// TODO: add database backend support so we don't need  stub version here
impl FromDatabase for Controls {
    #[cfg(feature = "sqlite")]
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

    #[cfg(not(feature = "sqlite"))]
    fn from_database(_database: &Database) -> Self {
        default()
    }
}

// TODO: Do this in a single transaction maybe? (don't know if it matters)
// TODO: add database backend support so we don't need  stub version here
impl ToDatabase for Controls {
    #[cfg(feature = "sqlite")]
    fn to_database(&self, database: &Database) -> Result<(), DatabaseError> {
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

    #[cfg(not(feature = "sqlite"))]
    fn to_database(&self, _database: &Database) -> Result<(), DatabaseError> {
        Ok(())
    }
}

impl IntoIterator for Controls {
    type Item = Keybind;
    type IntoIter = ControlsIter;

    fn into_iter(self) -> ControlsIter {
        ControlsIter {
            controls: self,
            current: Some(default()),
        }
    }
}

#[derive(Default)]
pub struct ControlsIter {
    controls: Controls,
    current: Option<Control>,
}

impl Iterator for ControlsIter {
    type Item = Keybind;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.and_then(|control| {
            let res = match control {
                Control::MoveUp => Keybind(Control::MoveUp, self.controls.move_up),
                Control::MoveDown => Keybind(Control::MoveDown, self.controls.move_down),
                Control::MoveLeft => Keybind(Control::MoveLeft, self.controls.move_left),
                Control::MoveRight => Keybind(Control::MoveRight, self.controls.move_right),
                Control::ZoomIn => Keybind(Control::ZoomIn, self.controls.zoom_in),
                Control::ZoomOut => Keybind(Control::ZoomOut, self.controls.zoom_out),
                Control::Pause => Keybind(Control::Pause, self.controls.pause),
                Control::Select => Keybind(Control::Select, self.controls.select),
            };

            self.current = control.next();

            Some(res)
        })
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Default, Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
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

const DEFAULT_UP_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowUp)),
    Some(Input::Keyboard(KeyCode::KeyW)),
];
const DEFAULT_DOWN_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowDown)),
    Some(Input::Keyboard(KeyCode::KeyS)),
];
const DEFAULT_LEFT_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowLeft)),
    Some(Input::Keyboard(KeyCode::KeyA)),
];
const DEFAULT_RIGHT_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowRight)),
    Some(Input::Keyboard(KeyCode::KeyD)),
];
const DEFAULT_ZOOM_IN_CONTROLS: InputList = [Some(Input::Keyboard(KeyCode::Comma)), None];
const DEFAULT_ZOOM_OUT_CONTROLS: InputList = [Some(Input::Keyboard(KeyCode::Period)), None];
const DEFAULT_PAUSE_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::Escape)),
    Some(Input::Keyboard(KeyCode::CapsLock)),
];
// TODO: Change this to mouse button left.
const DEFAULT_SELECT_CONTROLS: InputList = [Some(Input::Keyboard(KeyCode::KeyA)), None];

#[cfg(feature = "sqlite")]
fn query_keybind_or_set(database: &Database, keybind: &str, default: InputList) -> InputList {
    query_keybind_or_set_fallible(database, keybind, default)
        .inspect_err(|err| {
            warn!("Failed to get Keybind: '{keybind}' from sqlite with error: {err}")
        })
        .unwrap_or(default)
}

#[cfg(feature = "sqlite")]
fn query_keybind_or_set_fallible(
    database: &Database,
    keybind: &str,
    default: InputList,
) -> Result<InputList, sqlite::Error> {
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
fn query_keybind(database: &Database, keybind: &str) -> Result<Option<InputList>, sqlite::Error> {
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
        key1.map(|v| ron::from_str::<Input>(&v).unwrap()),
        key2.map(|v| ron::from_str::<Input>(&v).unwrap()),
    ]))
}

#[cfg(feature = "sqlite")]
fn set_keybind(database: &Database, keybind: &str, value: InputList) -> Result<(), sqlite::Error> {
    let query = "INSERT OR REPLACE INTO Keybinds VALUES (:keybind, :key1, :key2)";

    let values = [
        value[0].and_then(|v| ron::to_string(&v).ok()),
        value[1].and_then(|v| ron::to_string(&v).ok()),
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
