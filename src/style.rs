use crate::controls::Input;
use crate::embed_asset;
use crate::prelude::*;
use bevy::prelude::*;

const STYLE_DB_TABLE: &str = "Style";
const DEFAULT_FONT_PATH: &str = "embedded://assets/fonts/Ithaca/Ithaca-LVB75.ttf";
const DEFAULT_TEXT_COLOR: Color = Color::srgb_u8(0xe0, 0xde, 0xf4);
const BUTTON_SPRITE_IMAGE_PATH: &str = "embedded://assets/sprites/buttons.png";
const BUTTON_GLYPH_SIZE: UVec2 = UVec2::new(32, 36);
const BUTTON_GLYPH_TEXT_COLOR: Color = Color::BLACK;

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/fonts/Ithaca/Ithaca-LVB75.ttf");

        app.add_systems(Startup, add_style).add_systems(
            Update,
            sync_to_database.run_if(resource_exists_and_changed::<Style>),
        );
    }
}

fn sync_to_database(db: Res<Database>, style: Res<Style>, asset_server: Res<AssetServer>) {
    if let Err(err) = style.to_database(&db, &asset_server) {
        warn!("Failed to sync style settings to database with: {err}");
    };
}

pub fn add_style(mut commands: Commands, database: Res<Database>, asset_server: Res<AssetServer>) {
    commands.insert_resource(Style::from_database(
        database.into_inner(),
        asset_server.into_inner(),
    ));
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Style {
    pub font: Handle<Font>,
    icons: Icons,

    pub text_color: Color,
    //background_color: Color,
}

impl Style {
    pub fn font(&self, font_size: f32) -> TextFont {
        TextFont {
            font: self.font.clone(),
            font_size,
            ..default()
        }
    }

    /// Spawns Node(s) representing inputs, using glyphs where possible.
    pub fn display_keybind(&self, builder: &mut ChildSpawnerCommands<'_>, keybind: &Keybind) {
        let Keybind(control, key) = keybind;
        match key {
            [Some(a), Some(b)] => {
                builder
                    .spawn(Node { ..default() })
                    .with_children(move |builder| {
                        self.display_input(builder, a);
                        builder.spawn((
                            Text::new("/"),
                            self.font(32.0),
                            TextColor(self.text_color),
                            Label,
                            Pickable::IGNORE,
                        ));
                        self.display_input(builder, b);
                    });
            }
            [Some(a), None] | [None, Some(a)] => self.display_input(builder, a),
            [None, None] => {
                builder.spawn((
                    Text::new(format!("{control} Not Bound")),
                    self.font(32.0),
                    TextColor(self.text_color),
                    Label,
                    Pickable::IGNORE,
                ));
            }
        }
    }

    /// Spawns Node(s) representing inputs, using glyphs where possible.
    pub fn display_input(&self, builder: &mut ChildSpawnerCommands<'_>, input: &Input) {
        match input_index(input) {
            Some(index) => {
                if index == 0 {
                    builder.spawn((
                        Node {
                            height: Val::Px(BUTTON_GLYPH_SIZE.y as f32),
                            width: Val::Px(BUTTON_GLYPH_SIZE.x as f32),
                            padding: UiRect::px(0.0, 0.0, 0.0, 2.0),
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        self.icons.to_node(index),
                        Pickable::IGNORE,
                        children![(
                            Text::new(input.to_string()),
                            TextColor(BUTTON_GLYPH_TEXT_COLOR),
                            self.font(32.0),
                            Label,
                            Pickable::IGNORE,
                        )],
                    ));
                } else {
                    builder.spawn(self.icons.to_node(index));
                }
            }
            None => {
                builder.spawn((
                    Text::new(input.to_string()),
                    self.font(32.0),
                    TextColor(self.text_color),
                    Label,
                    Pickable::IGNORE,
                ));
            }
        }
    }

    /// Loads state from a database, resorting to defaults on failure.
    pub fn from_database(db: &Database, asset_server: &AssetServer) -> Self {
        let font_path: String =
            db.get_kv_table_direct_or_default(STYLE_DB_TABLE, "font", DEFAULT_FONT_PATH);

        Self {
            font: asset_server.load(font_path),
            icons: Icons::new(asset_server, BUTTON_SPRITE_IMAGE_PATH),

            text_color: db.get_kv_table_or_default(
                STYLE_DB_TABLE,
                "text_color",
                DEFAULT_TEXT_COLOR,
            ),
        }
    }

    /// Syncs data to the database
    pub fn to_database(
        &self,
        db: &Database,
        asset_server: &AssetServer,
    ) -> Result<(), crate::database::SetKvError> {
        let asset_path = asset_server
            .get_path(self.font.id())
            .expect("The font should have a file path!")
            .to_string();

        db.set_kv_table_direct(STYLE_DB_TABLE, "font", asset_path.as_str())?;
        db.set_kv_table(STYLE_DB_TABLE, "text_color", self.text_color)?;

        Ok(())
    }
}

#[derive(Reflect)]
pub struct Icons {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

impl Icons {
    pub fn new(asset_server: &AssetServer, path: &str) -> Self {
        let image = asset_server.load(path);

        let layout = TextureAtlasLayout::from_grid(
            BUTTON_GLYPH_SIZE,
            7,
            1,
            Some(UVec2::ZERO),
            Some(UVec2::ZERO),
        );
        let layout = asset_server.add(layout);

        Self { image, layout }
    }

    pub fn to_node(&self, index: usize) -> ImageNode {
        ImageNode {
            image: self.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: index,
            }),

            ..default()
        }
    }
}

/// All the of faint heart, look not upon here,
/// for it will only bring sorrow.
fn input_index(input: &Input) -> Option<usize> {
    use Input as I;
    use KeyCode as K;
    match input {
        // Single key icons
        I::Keyboard(
            K::Backquote
            | K::Backslash
            | K::BracketLeft
            | K::BracketRight
            | K::Comma
            | K::Digit0
            | K::Digit1
            | K::Digit2
            | K::Digit3
            | K::Digit4
            | K::Digit5
            | K::Digit6
            | K::Digit7
            | K::Digit8
            | K::Digit9
            | K::Equal
            | K::KeyA
            | K::KeyB
            | K::KeyC
            | K::KeyD
            | K::KeyE
            | K::KeyF
            | K::KeyG
            | K::KeyH
            | K::KeyI
            | K::KeyJ
            | K::KeyK
            | K::KeyL
            | K::KeyM
            | K::KeyN
            | K::KeyO
            | K::KeyP
            | K::KeyQ
            | K::KeyR
            | K::KeyS
            | K::KeyT
            | K::KeyU
            | K::KeyV
            | K::KeyW
            | K::KeyX
            | K::KeyY
            | K::KeyZ
            | K::Minus
            | K::Period
            | K::Quote
            | K::Semicolon
            | K::Slash
            | K::F1
            | K::F2
            | K::F3
            | K::F4
            | K::F5
            | K::F6
            | K::F7
            | K::F8
            | K::F9
            | K::F10
            | K::F11
            | K::F12
            | K::F13
            | K::F14
            | K::F15
            | K::F16
            | K::F17
            | K::F18
            | K::F19
            | K::F20
            | K::F21
            | K::F22
            | K::F23
            | K::F24
            | K::F25
            | K::F26
            | K::F27
            | K::F28
            | K::F29
            | K::F30
            | K::F31
            | K::F32
            | K::F33
            | K::F34
            | K::F35,
        ) => Some(0),
        I::Keyboard(K::ArrowLeft) => Some(1),
        I::Keyboard(K::ArrowRight) => Some(2),
        I::Keyboard(K::ArrowUp) => Some(3),
        I::Keyboard(K::ArrowDown) => Some(4),
        I::Keyboard(K::Tab) => Some(5),
        // All of the other keys. We should add some over time.
        I::Keyboard(
            K::Unidentified(_)
            | K::IntlBackslash
            | K::IntlRo
            | K::IntlYen
            | K::AltLeft
            | K::AltRight
            | K::Backspace
            | K::CapsLock
            | K::ContextMenu
            | K::ControlLeft
            | K::ControlRight
            | K::Enter
            | K::SuperLeft
            | K::SuperRight
            | K::ShiftLeft
            | K::ShiftRight
            | K::Space
            | K::Convert
            | K::KanaMode
            | K::Lang1
            | K::Lang2
            | K::Lang3
            | K::Lang4
            | K::Lang5
            | K::NonConvert
            | K::Delete
            | K::End
            | K::Help
            | K::Home
            | K::Insert
            | K::PageDown
            | K::PageUp
            | K::NumLock
            | K::Numpad0
            | K::Numpad1
            | K::Numpad2
            | K::Numpad3
            | K::Numpad4
            | K::Numpad5
            | K::Numpad6
            | K::Numpad7
            | K::Numpad8
            | K::Numpad9
            | K::NumpadAdd
            | K::NumpadBackspace
            | K::NumpadClear
            | K::NumpadClearEntry
            | K::NumpadComma
            | K::NumpadDecimal
            | K::NumpadDivide
            | K::NumpadEnter
            | K::NumpadEqual
            | K::NumpadHash
            | K::NumpadMemoryAdd
            | K::NumpadMemoryClear
            | K::NumpadMemoryRecall
            | K::NumpadMemoryStore
            | K::NumpadMemorySubtract
            | K::NumpadMultiply
            | K::NumpadParenLeft
            | K::NumpadParenRight
            | K::NumpadStar
            | K::NumpadSubtract
            | K::Escape
            | K::Fn
            | K::FnLock
            | K::PrintScreen
            | K::ScrollLock
            | K::Pause
            | K::BrowserBack
            | K::BrowserFavorites
            | K::BrowserForward
            | K::BrowserHome
            | K::BrowserRefresh
            | K::BrowserSearch
            | K::BrowserStop
            | K::Eject
            | K::LaunchApp1
            | K::LaunchApp2
            | K::LaunchMail
            | K::MediaPlayPause
            | K::MediaSelect
            | K::MediaStop
            | K::MediaTrackNext
            | K::MediaTrackPrevious
            | K::Power
            | K::Sleep
            | K::AudioVolumeDown
            | K::AudioVolumeMute
            | K::AudioVolumeUp
            | K::WakeUp
            | K::Meta
            | K::Hyper
            | K::Turbo
            | K::Abort
            | K::Resume
            | K::Suspend
            | K::Again
            | K::Copy
            | K::Cut
            | K::Find
            | K::Open
            | K::Paste
            | K::Props
            | K::Select
            | K::Undo
            | K::Hiragana
            | K::Katakana,
        ) => None,
        _ => None,
    }
}
