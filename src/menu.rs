use crate::controls::Control;
use crate::embed_asset;
use crate::prelude::*;
//use bevy::audio::Volume;
use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;

const FONT_PATH: &str = "embedded://assets/fonts/Ithaca/Ithaca-LVB75.ttf";

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/fonts/Ithaca/Ithaca-LVB75.ttf");
        app.init_state::<MenuState>()
            .add_systems(Startup, load_font)
            .add_systems(OnEnter(GameState::Menu), menu_screen_enter)
            .add_systems(OnEnter(MenuState::Main), main_enter)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMenuScreen>)
            .add_systems(OnEnter(MenuState::Settings), settings_enter)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnSettingsMenuScreen>,
            )
            .add_systems(OnEnter(MenuState::SettingsDisplay), display_settings_enter)
            .add_systems(
                OnExit(MenuState::SettingsDisplay),
                despawn_screen::<OnDisplaySettingsMenuScreen>,
            )
            .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_enter)
            .add_systems(
                OnExit(MenuState::SettingsSound),
                despawn_screen::<OnSoundSettingsMenuScreen>,
            )
            .add_systems(
                OnEnter(MenuState::SettingsControls),
                controls_settings_enter,
            )
            .add_systems(
                OnExit(MenuState::SettingsControls),
                despawn_screen::<OnControlsSettingsMenuScreen>,
            )
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(GameState::Menu)),
            )
            .add_systems(
                Update,
                controls_menu_action.run_if(in_state(MenuState::SettingsControls)),
            );
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Disabled,
    Main,
    Settings,
    SettingsDisplay,
    SettingsSound,
    SettingsControls,
}

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
struct OnSettingsMenuScreen;

#[derive(Component)]
struct OnDisplaySettingsMenuScreen;

#[derive(Component)]
struct OnSoundSettingsMenuScreen;

#[derive(Component)]
struct OnControlsSettingsMenuScreen;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    SettingsControls,
    SettingsDisplay,
    SettingsSound,
    BackToSettings,
    BackToMenu,
    Quit,
}

#[derive(Resource)]
struct Font(Handle<bevy::prelude::Font>);

fn load_font(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Font(assets.load(FONT_PATH)));
}

fn menu_screen_enter(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

const TITLE_COLOR: Color = Color::srgb_u8(0x26, 0x23, 0x3a);
const TEXT_COLOR: Color = Color::srgb_u8(0xe0, 0xde, 0xf4);
const NORMAL_BUTTON: Color = Color::srgb_u8(0x26, 0x23, 0x3a);
const PRESSED_BUTTON: Color = Color::srgb_u8(0x9c, 0xcf, 0xd8);
const HOVERED_BUTTON: Color = Color::srgb_u8(0x1f, 0x1d, 0x2e);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb_u8(0x1f, 0x1d, 0x2e);

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, Option::None) => HOVERED_BUTTON.into(),
            (Interaction::None, Option::None) => NORMAL_BUTTON.into(),
        }
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::SettingsControls => menu_state.set(MenuState::SettingsControls),
                MenuButtonAction::SettingsDisplay => menu_state.set(MenuState::SettingsDisplay),
                MenuButtonAction::SettingsSound => menu_state.set(MenuState::SettingsSound),
                MenuButtonAction::BackToSettings => menu_state.set(MenuState::Settings),
                MenuButtonAction::BackToMenu => menu_state.set(MenuState::Main),
            }
        }
    }
}

fn main_enter(mut commands: Commands, font: Res<Font>) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font: font.0.clone(),
        font_size: 33.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                // Display the game name
                (
                    Text::new("TCSS360 Project"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TITLE_COLOR),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),
                // Display three buttons for each action available from the main menu:
                // - new game
                // - settings
                // - quit
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                    children![
                        //(ImageNode::new(right_icon), button_icon_node.clone()),
                        (
                            Text::new("New Game"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Settings,
                    children![
                        //(ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Settings"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![
                        //(ImageNode::new(exit_icon), button_icon_node),
                        (Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),
                    ]
                ),
            ]
        )],
    ));
}

fn settings_enter(mut commands: Commands, font: Res<Font>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font: font.0.clone(),
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            Children::spawn(SpawnIter(
                [
                    (MenuButtonAction::SettingsControls, "Controls"),
                    (MenuButtonAction::SettingsDisplay, "Display"),
                    (MenuButtonAction::SettingsSound, "Sound"),
                    (MenuButtonAction::BackToMenu, "Back"),
                ]
                .into_iter()
                .map(move |(action, text)| {
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        action,
                        children![(Text::new(text), button_text_style.clone())],
                    )
                })
            ))
        )],
    ));
}

fn display_settings_enter(mut commands: Commands, font: Res<Font>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font: font.0.clone(),
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnDisplaySettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![(
                Button,
                button_node.clone(),
                BackgroundColor(NORMAL_BUTTON),
                MenuButtonAction::BackToSettings,
                children![(Text::new("Back"), button_text_style.clone())],
            )]
        )],
    ));
}

fn sound_settings_enter(mut commands: Commands, font: Res<Font> /*volume: Res<Volume>*/) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font: font.0.clone(),
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    //let button_node_clone = button_node.clone();
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSoundSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                //(
                //    Node {
                //        align_items: AlignItems::Center,
                //        ..default()
                //    },
                //    Children::spawn((
                //        Spawn((Text::new("Volume"), button_text_style.clone())),
                //        SpawnWith(move |parent: &mut ChildSpawner| {
                //            for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                //                let mut entity = parent.spawn((
                //                    Button,
                //                    Node {
                //                        width: Val::Px(30.0),
                //                        height: Val::Px(65.0),
                //                        ..button_node_clone.clone()
                //                    },
                //                    Volume(volume_setting),
                //                ));
                //                if volume == Volume(volume_setting) {
                //                    entity.insert(SelectedOption);
                //                }
                //            }
                //        })
                //    ))
                //),
                (
                    Button,
                    button_node,
                    MenuButtonAction::BackToSettings,
                    children![(Text::new("Back"), button_text_style)]
                )
            ]
        )],
    ));
}

#[derive(Component)]
enum ControlsButtonAction {
    SetControl(Control, usize),
    ClearControl(Control, usize),
    ResetControl(Control),
    ResetAll,
}

fn controls_settings_enter(
    mut commands: Commands,
    font: Res<Font>,
    controls: Res<Controls>,
    // volume: Res<Volume>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font: font.0.clone(),
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    //let button_node_clone = button_node.clone();
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnControlsSettingsMenuScreen,
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Children::spawn(SpawnIter(controls.clone().into_iter().map({
                    let bn = button_node.clone();
                    let bs = button_text_style.clone();
                    move |(control_name, control_bind)| {
                        (
                            Node {
                                width: Val::Percent(80.0),
                                height: Val::Px(65.0),
                                margin: UiRect::all(Val::Px(20.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Children::spawn(SpawnIter(
                                [
                                    (
                                        ControlsButtonAction::SetControl(control_name, 0),
                                        format!("{:?}", control_bind[0]),
                                    ),
                                    (
                                        ControlsButtonAction::ClearControl(control_name, 0),
                                        "Clear".to_string(),
                                    ),
                                    (
                                        ControlsButtonAction::SetControl(control_name, 1),
                                        format!("{:?}", control_bind[1]),
                                    ),
                                    (
                                        ControlsButtonAction::ClearControl(control_name, 1),
                                        "Clear".to_string(),
                                    ),
                                    (
                                        ControlsButtonAction::ResetControl(control_name),
                                        "Reset Both".to_string(),
                                    ),
                                ]
                                .into_iter()
                                .map({
                                    let bn = bn.clone();
                                    let bs = bs.clone();
                                    move |(action, text)| {
                                        (
                                            Button,
                                            bn.clone(),
                                            BackgroundColor(NORMAL_BUTTON),
                                            action,
                                            children![(Text::new(text), bs.clone())],
                                        )
                                    }
                                }),
                            )),
                        )
                    }
                })))
            ),
            (
                Button,
                button_node.clone(),
                BackgroundColor(NORMAL_BUTTON),
                ControlsButtonAction::ResetAll,
                children![(Text::new("ResetAll"), button_text_style.clone())],
            ),
            (
                Button,
                button_node.clone(),
                BackgroundColor(NORMAL_BUTTON),
                MenuButtonAction::BackToSettings,
                children![(Text::new("Back"), button_text_style.clone())],
            )
        ],
    ));
}

fn controls_menu_action(
    interaction_query: Query<
        (&Interaction, &ControlsButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut controls: ResMut<Controls>,
) {
    //pub fn set_control(&mut self, control: Control, entry: usize, bind: Option<KeyCode>) {
    for (interaction, contols_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match contols_button_action {
                ControlsButtonAction::SetControl(control, entry) => {
                    controls.set_control(*control, *entry, None)
                }
                ControlsButtonAction::ClearControl(control, entry) => {
                    controls.set_control(*control, *entry, None)
                }
                ControlsButtonAction::ResetControl(control) => controls.reset_control(*control),
                ControlsButtonAction::ResetAll => controls.reset_controls(),
            }
        }
    }
}

/// Helper method to despawn all of the entities with a given component.
/// This is used with the `On*` Components to easily destroy all of the components
/// on specific screens
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
