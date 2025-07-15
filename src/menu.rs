use crate::controls::{Control, keybind_to_string};
use crate::embed_asset;
use crate::prelude::*;
//use bevy::audio::Volume;
use bevy::{
    a11y::AccessibilityNode,
    ecs::{hierarchy::ChildSpawnerCommands, spawn::SpawnIter},
    input::mouse::{MouseScrollUnit, MouseWheel},
    input::{ButtonState, keyboard::KeyboardInput},
    picking::hover::HoverMap,
    prelude::*,
    ui::FocusPolicy,
};

use accesskit::{Node as Accessible, Role};

const FONT_PATH: &str = "embedded://assets/fonts/Ithaca/Ithaca-LVB75.ttf";

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/fonts/Ithaca/Ithaca-LVB75.ttf");
        app.register_system(control_prompt);
        app.init_state::<MenuState>()
            .init_state::<PromptControlState>()
            .add_systems(Startup, load_font)
            .add_systems(OnEnter(GameState::Menu), menu_screen_enter)
            .add_systems(OnEnter(MenuState::Main), main_enter)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMenuScreen>)
            .add_systems(OnEnter(MenuState::Settings), settings_enter)
            .add_systems(OnExit(MenuState::Settings), despawn_screen::<OnSettings>)
            .add_systems(OnEnter(MenuState::Display), display_enter)
            .add_systems(OnExit(MenuState::Display), despawn_screen::<OnDisplay>)
            .add_systems(OnEnter(MenuState::Sound), sound_enter)
            .add_systems(OnExit(MenuState::Sound), despawn_screen::<OnSoundScreen>)
            .add_systems(OnEnter(MenuState::Controls), controls_enter)
            .add_systems(
                Update,
                controls_changed.run_if(
                    in_state(MenuState::Controls)
                        .and(resource_changed::<Controls>.and(not(resource_added::<Controls>))),
                ),
            )
            .add_systems(
                Update,
                control_set_added
                    .run_if(in_state(MenuState::Controls).and(resource_added::<SetControlTarget>)),
            )
            .add_systems(
                Update,
                despawn_screen::<OnSetControl>.run_if(
                    in_state(MenuState::Controls).and(resource_removed::<SetControlTarget>),
                ),
            )
            .add_systems(OnExit(MenuState::Controls), despawn_screen::<OnControls>)
            .add_systems(
                Update,
                controls_menu_action.run_if(in_state(MenuState::Controls)),
            )
            .add_systems(
                Update,
                (update_scroll_position, menu_action, button_system)
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnEnter(PromptControlState::Single), control_prompt)
            .add_systems(
                OnExit(PromptControlState::Single),
                despawn_screen::<OnPromptControl>,
            )
            .add_systems(
                Update,
                assign_key_input.run_if(in_state(PromptControlState::Single)),
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

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum PromptControlState {
    #[default]
    Disabled,
    Single,
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
struct OnSetControl;

#[derive(Component)]
struct OnPromptControl;

#[derive(Component)]
struct CurrentControl;

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
const BACKGROUND_COLOR_SOLID: Color = Color::srgb_u8(0x26, 0x23, 0x3a);
const OVERLAY_COLOR: Color = Color::srgba_u8(0x26, 0x23, 0x3a, 0xdd);
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

fn display_enter(mut commands: Commands, font: Res<CurrentFont>) {
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

fn sound_enter(mut commands: Commands, font: Res<CurrentFont> /*volume: Res<Volume>*/) {
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
        TextLayout::new_with_justify(JustifyText::Center),
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
    Select(Control, usize),
    Reset(Control, usize),
    Set(Control, usize, KeyCode),
    Clear(Control, usize),
    Prompt(Control, usize),
    CancelPrompt,
    CancelSelect,
    ResetBoth(Control),
    ResetAll,
}

const NO_BLOCK_SCROLL: Pickable = Pickable {
    should_block_lower: false,
    is_hoverable: false,
};

fn controls_enter(mut commands: Commands, font: Res<CurrentFont>, controls: Res<Controls>) {
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
        TextLayout::new_with_justify(JustifyText::Center),
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
                    padding: UiRect::all(Val::Px(10.0)),

                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    row_gap: Val::Px(10.0),

                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|builder| {
                    controls.clone().into_iter().for_each(|(control, keys)| {
                        controls_row(builder, font.0.clone(), control, keys)
                    })
                });

            builder.spawn((
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
                FocusPolicy::Block,
                BackgroundColor(BACKGROUND_COLOR),
                children![
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::Settings,
                        children![(Text::new("Back"), button_text_style.clone())],
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        ControlsButtonAction::ResetAll,
                        children![(Text::new("ResetAll"), button_text_style.clone())],
                    ),
                ],
            ));
        });
}

fn controls_row(
    builder: &mut ChildSpawnerCommands<'_>,
    font: Handle<Font>,
    control: Control,
    keys: Keybind,
) {
    builder
        .spawn((Node::default(), NO_BLOCK_SCROLL))
        .with_children(|builder| {
            builder
                .spawn((
                    Node {
                        width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Label,
                    AccessibilityNode(Accessible::new(Role::ListItem)),
                    NO_BLOCK_SCROLL,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(control.to_string()),
                        TextColor(TITLE_COLOR),
                        TextFont {
                            font: font.clone(),
                            font_size: 33.0,
                            ..default()
                        },
                        NO_BLOCK_SCROLL,
                    ));
                });

            [
                (
                    keybind_to_string(keys[0]),
                    ControlsButtonAction::Select(control, 0),
                    Val::Px(150.0),
                ),
                (
                    keybind_to_string(keys[1]),
                    ControlsButtonAction::Select(control, 1),
                    Val::Px(150.0),
                ),
                (
                    "Reset Both".into(),
                    ControlsButtonAction::ResetBoth(control),
                    Val::Px(125.0),
                ),
            ]
            .into_iter()
            .for_each(|(name, action, width)| {
                controls_button(builder, font.clone(), name, action, width)
            })
        });
}

fn controls_button(
    builder: &mut ChildSpawnerCommands<'_>,
    font: Handle<Font>,
    name: String,
    action: ControlsButtonAction,
    width: Val,
) {
    let button_node = Node {
        margin: UiRect::px(2.0, 2.0, 0.0, 0.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        overflow: Overflow::clip(),
        width,
        ..default()
    };

    builder
        .spawn((
            Button,
            button_node.clone(),
            action,
            AccessibilityNode(Accessible::new(Role::ListItem)),
            NO_BLOCK_SCROLL,
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(name),
                TextFont {
                    font: font.clone(),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Label,
                NO_BLOCK_SCROLL,
            ));
        });
}

fn controls_menu_action(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &ControlsButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut controls: ResMut<Controls>,
) {
    for (interaction, contols_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match contols_button_action {
                ControlsButtonAction::Select(control, entry) => {
                    commands.set_state(PromptControlState::Disabled);
                    commands.insert_resource(SetControlTarget(*control, *entry));
                }
                ControlsButtonAction::CancelSelect => {
                    commands.remove_resource::<SetControlTarget>();
                }
                ControlsButtonAction::Clear(control, idx) => {
                    controls.set_control(*control, *idx, None);
                }
                ControlsButtonAction::Reset(control, idx) => {
                    controls.reset_control_part(*control, *idx);
                }
                ControlsButtonAction::Prompt(control, idx) => {
                    commands.set_state(PromptControlState::Single);
                }
                ControlsButtonAction::CancelPrompt => {
                    commands.run_system_cached(despawn_screen::<OnPromptControl>);
                }
                ControlsButtonAction::Set(control, idx, code) => {
                    controls.set_control(*control, *idx, Some(*code));
                }
                ControlsButtonAction::ResetBoth(control) => controls.reset_control(*control),
                ControlsButtonAction::ResetAll => controls.reset_controls(),
            }
        }
    }
}

fn controls_changed(
    controls: Res<Controls>,
    current_control: Res<SetControlTarget>,
    button: Query<(&ControlsButtonAction, &Children)>,
    mut current: Query<&mut Text, With<CurrentControl>>,
    mut text_query: Query<&mut Text, Without<CurrentControl>>,
) {
    if let Ok(mut current) = current.single_mut() {
        *current = Text::new(format!(
            "Current: '{}'",
            keybind_to_string(controls.get_control(current_control.0, current_control.1))
        ));
    };

    for (action, children) in button.iter() {
        match action {
            ControlsButtonAction::Select(control, idx)
            | ControlsButtonAction::Set(control, idx, ..) => {
                let mut text = text_query.get_mut(children[0]).unwrap();

                **text = keybind_to_string(controls.get_control(*control, *idx));
            }
            ControlsButtonAction::CancelSelect
            | ControlsButtonAction::CancelPrompt
            | ControlsButtonAction::Prompt(..)
            | ControlsButtonAction::Reset(..)
            | ControlsButtonAction::ResetBoth(..)
            | ControlsButtonAction::Clear(..)
            | ControlsButtonAction::ResetAll => {}
        }
    }
}

const CONTROLS_LINE_HEIGHT: f32 = 65.0;

fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * CONTROLS_LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}

fn control_prompt(mut commands: Commands, font: Res<CurrentFont>, target: Res<SetControlTarget>) {
    let SetControlTarget(target_control, target_idx) = *target;
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
        TextLayout::new_with_justify(JustifyText::Center),
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
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            FocusPolicy::Block,
            OnPromptControl,
            BackgroundColor(BACKGROUND_COLOR_SOLID),
            ZIndex(2),
        ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    Text::new("Press any key to bind,\n or click 'Cancel'"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),],
            ));

            builder.spawn((
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
                children![(
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    ControlsButtonAction::CancelPrompt,
                    children![(Text::new("Cancel"), button_text_style.clone())],
                )],
            ));
        });
}

#[derive(Resource)]
struct SetControlTarget(Control, usize);

fn control_set_added(
    mut commands: Commands,
    font: Res<CurrentFont>,
    controls: Res<Controls>,
    target: Res<SetControlTarget>,
) {
    let SetControlTarget(target_control, target_idx) = *target;
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
        TextLayout::new_with_justify(JustifyText::Center),
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
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            FocusPolicy::Block,
            OnSetControl,
            BackgroundColor(OVERLAY_COLOR),
            ZIndex(1),
        ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    width: Val::Percent(95.0),
                    height: Val::Percent(85.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Text::new(format!(
                            "Rebind '{target_control}' {}",
                            (target_idx as u8 + b'A') as char
                        )),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ),
                    (
                        CurrentControl,
                        Text::new(format!(
                            "Current: '{}'",
                            keybind_to_string(controls.get_control(target_control, target_idx))
                        )),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        ControlsButtonAction::Prompt(target_control, target_idx),
                        children![(Text::new("Enter"), button_text_style.clone(),),],
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        ControlsButtonAction::Clear(target_control, target_idx),
                        children![(Text::new("Clear"), button_text_style.clone(),),],
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        ControlsButtonAction::Reset(target_control, target_idx),
                        children![(Text::new("Reset"), button_text_style.clone(),),],
                    )
                ],
            ));

            builder.spawn((
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
                children![(
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    ControlsButtonAction::CancelSelect,
                    children![(Text::new("Cancel"), button_text_style.clone())],
                )],
            ));
        });
}

fn assign_key_input(
    mut kb_ev: EventReader<KeyboardInput>,
    mut controls: ResMut<Controls>,
    target: Res<SetControlTarget>,
    mut commands: Commands,
) {
    for ev in kb_ev.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls.set_control(target.0, target.1, Some(ev.key_code));
                commands.set_state(PromptControlState::Disabled);
            }
            ButtonState::Released => {}
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
