use crate::controls::{Control, keybind_to_string};
use crate::embed_asset;
use crate::prelude::*;
//use bevy::audio::Volume;
use bevy::{
    a11y::AccessibilityNode,
    ecs::{relationship::RelatedSpawnerCommands, spawn::SpawnIter},
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};

use accesskit::{Node as Accessible, Role};

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
            .add_systems(OnExit(MenuState::Settings), despawn_screen::<OnSettings>)
            .add_systems(OnEnter(MenuState::Display), display_settings_enter)
            .add_systems(OnExit(MenuState::Display), despawn_screen::<OnDisplay>)
            .add_systems(OnEnter(MenuState::Sound), sound_settings_enter)
            .add_systems(OnExit(MenuState::Sound), despawn_screen::<OnSoundScreen>)
            .add_systems(OnEnter(MenuState::Controls), controls_settings_enter)
            .add_systems(OnExit(MenuState::Controls), despawn_screen::<OnControls>)
            .add_systems(
                Update,
                (controls_menu_action, update_scroll_position)
                    .run_if(in_state(MenuState::Controls)),
            )
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(GameState::Menu)),
            );
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Disabled,
    Main,
    Settings,
    Display,
    Sound,
    Controls,
}

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
struct OnSettings;

#[derive(Component)]
struct OnDisplay;

#[derive(Component)]
struct OnSoundScreen;

#[derive(Component)]
struct OnControls;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    MainMenu,
    Settings,
    Controls,
    Display,
    Sound,
    Quit,
}

#[derive(Resource)]
struct CurrentFont(Handle<Font>);

fn load_font(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(CurrentFont(assets.load(FONT_PATH)));
}

fn menu_screen_enter(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

const BACKGROUND_COLOR: Color = Color::srgba_u8(0x26, 0x23, 0x3a, 0xaa);
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
                MenuButtonAction::Controls => menu_state.set(MenuState::Controls),
                MenuButtonAction::Display => menu_state.set(MenuState::Display),
                MenuButtonAction::Sound => menu_state.set(MenuState::Sound),
                MenuButtonAction::MainMenu => menu_state.set(MenuState::Main),
            }
        }
    }
}

fn main_enter(mut commands: Commands, font: Res<CurrentFont>) {
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

fn settings_enter(mut commands: Commands, font: Res<CurrentFont>) {
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
        OnSettings,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            Children::spawn(SpawnIter(
                [
                    (MenuButtonAction::Controls, "Controls"),
                    (MenuButtonAction::Display, "Display"),
                    (MenuButtonAction::Sound, "Sound"),
                    (MenuButtonAction::MainMenu, "Back"),
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

fn display_settings_enter(mut commands: Commands, font: Res<CurrentFont>) {
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
        OnDisplay,
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
                MenuButtonAction::Settings,
                children![(Text::new("Back"), button_text_style.clone())],
            )]
        )],
    ));
}

fn sound_settings_enter(
    mut commands: Commands,
    font: Res<CurrentFont>, /*volume: Res<Volume>*/
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
        OnSoundScreen,
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
                    MenuButtonAction::Settings,
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

const CONTROLS_GRID_WIDTH: u16 = 8;

fn controls_settings_enter(
    mut commands: Commands,
    font: Res<CurrentFont>,
    controls: Res<Controls>,
    // volume: Res<Volume>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(5.0)),
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
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnControls,
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(85.0),
                    margin: UiRect::all(Val::Px(10.0)),

                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    row_gap: Val::Px(10.0),

                    grid_template_columns: RepeatedGridTrack::flex(CONTROLS_GRID_WIDTH, 5.0),
                    display: Display::Grid,

                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|builder| {
                    controls.clone().into_iter().for_each(|(control, keys)| {
                        controls_settings_row(builder, font.0.clone(), control, keys)
                    })
                });

            builder
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        align_self: AlignSelf::End,
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        ControlsButtonAction::ResetAll,
                        children![(Text::new("ResetAll"), button_text_style.clone())],
                    ));
                    builder.spawn((
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::Settings,
                        children![(Text::new("Back"), button_text_style.clone())],
                    ));
                });
        });
}

fn controls_settings_row(
    builder: &mut RelatedSpawnerCommands<'_, ChildOf>,
    font: Handle<Font>,
    control: Control,
    keys: Keybind,
) {
    builder
        .spawn((
            Node {
                width: Val::Vw(100.0 / CONTROLS_GRID_WIDTH as f32),
                min_height: Val::Px(60.0),
                align_items: AlignItems::Center,
                ..default()
            },
            Label,
            AccessibilityNode(Accessible::new(Role::ListItem)),
        ))
        .insert(Pickable {
            should_block_lower: false,
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                Text::new(control.to_string()),
                TextColor(TITLE_COLOR),
                TextFont {
                    font: font.clone(),
                    font_size: 33.0,
                    ..default()
                },
            ));
        });

    [
        (
            keybind_to_string(keys[0]),
            ControlsButtonAction::SetControl(control, 0),
            GridPlacement::span(2),
        ),
        (
            "Clear".into(),
            ControlsButtonAction::ClearControl(control, 0),
            GridPlacement::span(1),
        ),
        (
            keybind_to_string(keys[1]),
            ControlsButtonAction::SetControl(control, 1),
            GridPlacement::span(2),
        ),
        (
            "Clear".into(),
            ControlsButtonAction::ClearControl(control, 1),
            GridPlacement::span(1),
        ),
        (
            "Reset".into(),
            ControlsButtonAction::ResetControl(control),
            GridPlacement::span(1),
        ),
    ]
    .into_iter()
    .for_each(|(name, action, grid)| {
        controls_settings_button(builder, font.clone(), name, action, grid)
    })
}

fn controls_settings_button(
    builder: &mut RelatedSpawnerCommands<'_, ChildOf>,
    font: Handle<Font>,
    name: Box<str>,
    action: ControlsButtonAction,
    grid_column: GridPlacement,
) {
    let button_node = Node {
        width: Val::Vw(100.0 / CONTROLS_GRID_WIDTH as f32 * grid_column.get_span().unwrap() as f32),
        height: Val::Px(CONTROLS_LINE_HEIGHT),
        margin: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(0.0), Val::Px(0.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        overflow: Overflow::clip(),
        grid_column,
        ..default()
    };

    builder
        .spawn((Button, button_node.clone(), action))
        .insert(Pickable {
            should_block_lower: false,
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn((
                    Text::new(name),
                    TextFont {
                        font: font.clone(),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                ))
                .insert(Pickable {
                    should_block_lower: false,
                    ..default()
                });
        });
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

const CONTROLS_LINE_HEIGHT: f32 = 65.0;

pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (
                mouse_wheel_event.x * CONTROLS_LINE_HEIGHT,
                mouse_wheel_event.y * CONTROLS_LINE_HEIGHT,
            ),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
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
