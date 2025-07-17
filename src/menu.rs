//! TODO: Make the UI hexagon based.

use crate::controls::Control;
use crate::controls::{Input, Keybind, input_to_screen};
use crate::embed_asset;
use crate::prelude::*;

use bevy::{
    a11y::AccessibilityNode,
    ecs::{hierarchy::ChildSpawnerCommands, spawn::SpawnIter},
    input::{
        ButtonState,
        gamepad::GamepadButtonChangedEvent,
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseScrollUnit},
    },
    picking::hover::HoverMap,
    prelude::*,
    ui::FocusPolicy,
};

use accesskit::{Node as Accessible, Role};

const FONT_PATH: &str = "embedded://assets/fonts/Ithaca/Ithaca-LVB75.ttf";
const BACKGROUND_COLOR: Color = Color::srgba_u8(0x26, 0x23, 0x3a, 0xaa);
const BACKGROUND_COLOR_SOLID: Color = Color::srgb_u8(0x26, 0x23, 0x3a);
const TITLE_COLOR: Color = Color::srgb_u8(0x26, 0x23, 0x3a);
const TEXT_COLOR: Color = Color::srgb_u8(0xe0, 0xde, 0xf4);
const NORMAL_BUTTON: Color = Color::srgb_u8(0x26, 0x23, 0x3a);
const PRESSED_BUTTON: Color = Color::srgb_u8(0x9c, 0xcf, 0xd8);
const HOVERED_BUTTON: Color = Color::srgb_u8(0x1f, 0x1d, 0x2e);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb_u8(0x1f, 0x1d, 0x2e);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/fonts/Ithaca/Ithaca-LVB75.ttf");

        app.init_state::<MenuState>()
            .add_systems(Startup, load_font)
            .add_systems(
                Update,
                (menu_action, button_highlight).run_if(in_state(GameState::Menu)),
            )
            .add_systems(Update, escape_out.run_if(in_state(GameState::Menu)))
            .add_systems(OnEnter(GameState::Menu), menu_screen_enter)
            .add_systems(OnEnter(MenuState::Main), main_enter)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMenuScreen>)
            .add_systems(OnEnter(MenuState::Settings), settings_enter)
            .add_systems(OnExit(MenuState::Settings), despawn_screen::<OnSettings>)
            .add_systems(OnEnter(MenuState::Display), display_enter)
            .add_systems(OnExit(MenuState::Display), despawn_screen::<OnDisplay>)
            .add_systems(OnEnter(MenuState::Sound), sound_enter)
            .add_systems(OnExit(MenuState::Sound), despawn_screen::<OnSoundScreen>)
            .add_systems(
                OnTransition {
                    exited: MenuState::Settings,
                    entered: MenuState::Controls,
                },
                controls_enter,
            )
            .add_systems(
                OnTransition {
                    exited: MenuState::Controls,
                    entered: MenuState::Settings,
                },
                despawn_screen::<OnControls>,
            )
            .add_systems(OnEnter(MenuState::ControlPrompt), control_prompt_enter)
            .add_systems(
                OnExit(MenuState::ControlPrompt),
                (
                    despawn_screen::<OnControlPrompt>,
                    remove_control_prompt_target,
                ),
            )
            .add_systems(
                Update,
                controls_changed.run_if(
                    in_state(MenuState::Controls).and(resource_exists_and_changed::<Controls>),
                ),
            )
            .add_systems(
                Update,
                assign_key_input.run_if(in_state(MenuState::ControlPrompt)),
            );
    }
}

#[derive(Resource)]
struct CurrentFont(Handle<Font>);

#[derive(Resource)]
struct ControlPromptTarget(Control, usize);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Disabled,
    Main,
    Settings,
    Display,
    Sound,
    Controls,
    ControlPrompt,
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
struct OnControlPrompt;

/// Specifies the action that should be taken the button it is on is clicked.
///
/// The node will need to be observed by `menu_button_action` for this to take effect.
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

/// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

fn load_font(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(CurrentFont(assets.load(FONT_PATH)));
}

fn menu_screen_enter(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn escape_out(mut commands: Commands, menu_state: Res<State<MenuState>>, key: Res<InputState>) {
    if key.just_pressed(Control::Pause) {
        use MenuState as M;
        match *menu_state.get() {
            // TODO: Implement title screen and pausing separately.
            M::Disabled | M::Main => {}
            M::Settings => commands.set_state(MenuState::Main),
            M::Sound | M::Display | M::Controls => commands.set_state(MenuState::Settings),
            M::ControlPrompt => {}
        }
    }
}

fn button_highlight(
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
                    menu_state.set(MenuState::Disabled);
                    game_state.set(GameState::Game);
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

#[derive(Component, Clone)]
enum ControlsButtonAction {
    Prompt(Control, usize),
    PromptCancel,
    ResetBoth(Control),
    ResetAll,
}

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
                .observe(update_scroll_position_event)
                .with_children(|builder| {
                    controls
                        .clone()
                        .into_iter()
                        .for_each(|keybind| controls_row(builder, font.0.clone(), keybind))
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
                    FocusPolicy::Block,
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::Settings,
                        children![(Text::new("Back"), button_text_style.clone())],
                    ));
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            ControlsButtonAction::ResetAll,
                            children![(Text::new("Reset All"), button_text_style.clone())],
                        ))
                        .observe(controls_menu_click);
                    builder.spawn((
                        Text::new(
                            "Note: The keys show are based on the physical key and may not reflect the keyboard input in a text box.",
                        ),
                        (
                            TextFont {
                                font: font.0.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            TextLayout::new_with_justify(JustifyText::Center),
                        ),
                        Pickable::IGNORE,
                    ));
                });
        });
}

fn controls_row(builder: &mut ChildSpawnerCommands<'_>, font: Handle<Font>, keybind: Keybind) {
    let Keybind(control, keys) = keybind;
    builder
        .spawn((Node::default(), Pickable::IGNORE))
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
                    Pickable::IGNORE,
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
                        Pickable::IGNORE,
                    ));
                });

            for (i, key) in keys.into_iter().enumerate() {
                builder
                    .spawn((
                        Button,
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Px(150.0),
                            margin: UiRect::px(2.0, 2.0, 0.0, 0.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        ControlsButtonAction::Prompt(control, i),
                        AccessibilityNode(Accessible::new(Role::ListItem)),
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: true,
                        },
                    ))
                    .observe(controls_menu_click)
                    .with_children(|builder| input_to_screen(font.clone(), builder, &key));
            }

            builder
                .spawn((
                    Button,
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Px(150.0),
                        margin: UiRect::px(2.0, 2.0, 0.0, 0.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    ControlsButtonAction::ResetBoth(control),
                    AccessibilityNode(Accessible::new(Role::ListItem)),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                    children![(
                        Text("Reset Both".into()),
                        TextFont {
                            font: font.clone(),
                            font_size: 33.0,
                            ..default()
                        },
                    )],
                ))
                .observe(controls_menu_click);
        });
}

fn controls_menu_click(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut controls: ResMut<Controls>,
    target_query: Query<&ControlsButtonAction>,
) {
    if let Ok(action) = target_query.get(click.target()) {
        use ControlsButtonAction as C;
        use PointerButton as P;
        match (click.button, action) {
            (P::Primary, C::Prompt(control, entry)) => {
                commands.insert_resource(ControlPromptTarget(*control, *entry));
                commands.set_state(MenuState::ControlPrompt);
            }
            (P::Secondary, C::Prompt(control, entry)) => {
                controls.set_control(*control, *entry, None)
            }
            (P::Middle, C::Prompt(control, entry)) => controls.reset_control_part(*control, *entry),

            (P::Primary, C::PromptCancel) => commands.set_state(MenuState::Controls),
            (_, C::PromptCancel) => {}

            (P::Primary, ControlsButtonAction::ResetBoth(control)) => {
                controls.reset_control(*control)
            }
            (_, C::ResetBoth(..)) => {}

            (P::Primary, ControlsButtonAction::ResetAll) => controls.reset_controls(),
            (_, C::ResetAll) => {}
        }
    }
    click.propagate(false);
}

fn controls_changed(
    mut commands: Commands,
    font: Res<CurrentFont>,
    controls: Res<Controls>,
    button: Query<(Entity, &ControlsButtonAction, &Children)>,
) {
    for (entity, action, children) in button.iter() {
        use ControlsButtonAction as C;
        match action {
            C::Prompt(control, idx) => {
                let key = controls.get_control_part(*control, *idx);
                for child in children {
                    if let Ok(mut child) = commands.get_entity(*child) {
                        child.despawn();
                    }
                }
                commands
                    .get_entity(entity)
                    .expect("It was just clicked, it should be alive?")
                    .remove_children(children)
                    .with_children(|builder| input_to_screen(font.0.clone(), builder, &key));
            }
            C::PromptCancel | C::ResetBoth(..) | C::ResetAll => {}
        }
    }
}

fn remove_control_prompt_target(mut commands: Commands) {
    commands.remove_resource::<ControlPromptTarget>();
}

fn control_prompt_enter(mut commands: Commands, font: Res<CurrentFont>) {
    let button_text_style = (
        TextFont {
            font: font.0.clone(),
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        TextLayout::new_with_justify(JustifyText::Center),
    );

    commands.spawn((
        Node {
            display: Display::Flex,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        FocusPolicy::Block,
        OnControlPrompt,
        BackgroundColor(BACKGROUND_COLOR_SOLID),
        ZIndex(2),
        children![(
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    Text::new("Press any key to bind,"),
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
                    Text::new("or click 'Cancel'"),
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
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    ControlsButtonAction::PromptCancel,
                    children![(
                        Text::new("Cancel"),
                        button_text_style.clone(),
                        ControlsButtonAction::PromptCancel
                    )],
                )
            ],
        )],
    ));
}

fn assign_key_input(
    mut commands: Commands,
    mut keyboard: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut gamepad: EventReader<GamepadButtonChangedEvent>,
    mut controls: ResMut<Controls>,
    cancel_button_query: Query<(), With<ControlsButtonAction>>,
    target: Res<ControlPromptTarget>,
    hover_map: Res<HoverMap>,
) {
    for ev in keyboard.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls.set_control(target.0, target.1, Some(Input::Keyboard(ev.key_code)));
                commands.set_state(MenuState::Controls);
                return;
            }
            ButtonState::Released => {}
        }
    }

    for ev in mouse.read() {
        match ev.state {
            ButtonState::Pressed => {
                if ev.button == MouseButton::Left {
                    for (_pointer, pointer_map) in hover_map.iter() {
                        for (entity, _hit) in pointer_map.iter() {
                            if let Ok(_) = cancel_button_query.get(*entity) {
                                commands.set_state(MenuState::Controls);
                                return;
                            }
                        }
                    }
                }

                controls.set_control(target.0, target.1, Some(Input::Mouse(ev.button)));
                commands.set_state(MenuState::Controls);
                return;
            }
            ButtonState::Released => {}
        }
    }

    for ev in gamepad.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls.set_control(target.0, target.1, Some(Input::Gamepad(ev.button)));
                commands.set_state(MenuState::Controls);
                return;
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

const CONTROLS_LINE_HEIGHT: f32 = 65.0;

fn update_scroll_position_event(
    mut trigger: Trigger<Pointer<Scroll>>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    let mut target = scrolled_node_query
        .get_mut(trigger.target)
        .expect("Cannot scroll a non-scrollable entity");

    let event = trigger.event();
    let dy = match event.unit {
        MouseScrollUnit::Line => event.y * CONTROLS_LINE_HEIGHT,
        MouseScrollUnit::Pixel => event.y,
    };

    target.offset_y -= dy;

    trigger.propagate(false);
}
