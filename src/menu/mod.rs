//! TODO: Make the UI hexagon based.

use crate::controls::Control;
use crate::controls::{Input, Keybind, input_to_screen};
use crate::prelude::*;

use bevy::{
    a11y::AccessibilityNode,
    ecs::hierarchy::ChildSpawnerCommands,
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
        app.init_state::<MenuState>()
            .add_systems(Update, (button_highlight).run_if(in_state(GameState::Menu)))
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
                (controls_enter, controls_wip_insert),
            )
            .add_systems(
                OnTransition {
                    exited: MenuState::Controls,
                    entered: MenuState::Settings,
                },
                despawn_screen::<OnControls>,
            )
            .add_systems(
                Update,
                controls_changed.run_if(
                    in_state(MenuState::Controls).and(resource_exists_and_changed::<ControlsWIP>),
                ),
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
                assign_key_input.run_if(in_state(MenuState::ControlPrompt)),
            )
            .add_systems(
                OnEnter(MenuState::ControlSaveWarning),
                control_save_warning_enter,
            )
            .add_systems(
                OnExit(MenuState::ControlSaveWarning),
                (
                    despawn_screen::<OnControlSaveWarning>,
                    remove_resource::<ControlsWIP>,
                ),
            )
            .add_systems(
                OnTransition {
                    exited: MenuState::ControlSaveWarning,
                    entered: MenuState::Settings,
                },
                despawn_screen::<OnControls>,
            );
    }
}

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
    ControlSaveWarning,
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

#[derive(Component)]
struct OnControlSaveWarning;

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

fn menu_screen_enter(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn escape_out(
    menu_state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    controls_master: Res<Controls>,
    controls_wip: Option<Res<ControlsWIP>>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
        use MenuState as M;
        match *menu_state.get() {
            // TODO: Implement title screen and pausing separately.
            M::Disabled | M::Main => {}
            M::Settings => next_state.set(MenuState::Main),
            M::Sound | M::Display | M::ControlSaveWarning => next_state.set(MenuState::Settings),
            M::ControlPrompt => {
                // don't do anything, it should be caught by [`assign_key_input`]
            }
            M::Controls => {
                if controls_wip.unwrap().0 == *controls_master {
                    next_state.set(MenuState::Settings);
                } else {
                    next_state.set(MenuState::ControlSaveWarning);
                }
            }
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

fn menu_button_click(
    mut click: Trigger<Pointer<Click>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    target_query: Query<&MenuButtonAction>,
) {
    if click.button == PointerButton::Primary {
        let Ok(menu_button_action) = target_query.get(click.target()) else {
            return;
        };
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

    click.propagate(false);
}

fn main_enter(mut commands: Commands, style: Res<Style>) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = style.font(33.0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMenuScreen,
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    // Display the game name
                    builder.spawn((
                        Text::new("A Hex Befalls\nThe Hexagons"),
                        style.font(67.0),
                        TextColor(TITLE_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                    // Display three buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    builder
                        .spawn((
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
                                    Pickable::IGNORE
                                ),
                            ],
                        ))
                        .observe(menu_button_click);
                    builder
                        .spawn((
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
                                    Pickable::IGNORE
                                ),
                            ],
                        ))
                        .observe(menu_button_click);
                    builder
                        .spawn((
                            Button,
                            button_node,
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Quit,
                            children![
                                //(ImageNode::new(exit_icon), button_icon_node),
                                (
                                    Text::new("Quit"),
                                    button_text_font,
                                    TextColor(TEXT_COLOR),
                                    Pickable::IGNORE
                                ),
                            ],
                        ))
                        .observe(menu_button_click);
                });
        });
}

fn settings_enter(mut commands: Commands, style: Res<Style>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (style.font(33.0), TextColor(TEXT_COLOR));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSettings,
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    [
                        (MenuButtonAction::Controls, "Controls"),
                        (MenuButtonAction::Display, "Display"),
                        (MenuButtonAction::Sound, "Sound"),
                        (MenuButtonAction::MainMenu, "Back"),
                    ]
                    .into_iter()
                    .for_each(|(action, text)| {
                        builder
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                action,
                                children![(
                                    Text::new(text),
                                    button_text_style.clone(),
                                    Pickable::IGNORE
                                )],
                            ))
                            .observe(menu_button_click);
                    });
                });
        });
}

fn display_enter(mut commands: Commands, style: Res<Style>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (style.font(33.0), TextColor(TEXT_COLOR));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnDisplay,
        ))
        .with_children(|builder| {
            builder
                .spawn(
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                )
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Settings,
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(menu_button_click);
                });
        });
}

fn sound_enter(mut commands: Commands, style: Res<Style> /*volume: Res<Volume>*/) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        style.font(33.0),
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(TEXT_COLOR),
    );

    //let button_node_clone = button_node.clone();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSoundScreen,
        ))
        .with_children(|builder| {
            builder
                .spawn(
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                )
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Settings,
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(menu_button_click);
                });
        });
}

#[derive(Component, Clone, Debug)]
enum ControlsButtonAction {
    Prompt(Control, usize),
    PromptCancel,
    ResetBoth(Control),
    ResetAll,
    Save,
    Discard,
    Back,
}

fn controls_enter(mut commands: Commands, style: Res<Style>, controls: Res<Controls>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        style.font(33.0),
        TextColor(style.text_color),
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
                        .for_each(|keybind| controls_row(builder, &style, keybind))
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
                        ControlsButtonAction::Back,
                        children![(Text::new("Back"), button_text_style.clone(), Pickable::IGNORE)],
                    ))
                        .observe(controls_menu_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            ControlsButtonAction::Save,
                            children![(Text::new("Save"), button_text_style.clone())],
                        ))
                        .observe(controls_menu_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            ControlsButtonAction::Discard,
                            children![(Text::new("Discard"), button_text_style.clone())],
                        ))
                        .observe(controls_menu_click);

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
                            style.font(18.0),
                            TextLayout::new_with_justify(JustifyText::Center),
                        ),
                        Pickable::IGNORE,
                    ));
                });
        });
}

fn controls_row(builder: &mut ChildSpawnerCommands<'_>, style: &Style, keybind: Keybind) {
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
                        style.font(33.0),
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
                    .with_children(|builder| input_to_screen(style, builder, &key));
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
                        style.font(33.0),
                        TextColor(TEXT_COLOR)
                    )],
                ))
                .observe(controls_menu_click);
        });
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
struct ControlsWIP(Controls);

fn controls_wip_insert(mut commands: Commands, controls_master: Res<Controls>) {
    commands.insert_resource(ControlsWIP(controls_master.clone()));
}

fn controls_menu_click(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut controls_master: ResMut<Controls>,
    mut controls_wip: ResMut<ControlsWIP>,
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
                controls_wip.0.set_control(*control, *entry, None);
            }
            (P::Middle, C::Prompt(control, entry)) => {
                controls_wip.0.reset_control_part(*control, *entry);
            }
            (P::Primary, C::PromptCancel) => commands.set_state(MenuState::Controls),
            (_, C::PromptCancel) => {}

            (P::Primary, C::ResetBoth(control)) => {
                controls_wip.0.reset_control(*control);
            }
            (_, C::ResetBoth(..)) => {}

            (P::Primary, C::ResetAll) => {
                controls_wip.0.reset_controls();
            }
            (_, C::ResetAll) => {}

            (P::Primary, C::Save) => {
                *controls_master = controls_wip.0.clone();
            }
            (_, C::Save) => {}

            (P::Primary, C::Discard) => {
                controls_wip.0 = controls_master.clone();
            }
            (_, C::Discard) => {}

            (P::Primary, C::Back) => {
                if controls_wip.0 == *controls_master {
                    commands.set_state(MenuState::Settings);
                } else {
                    commands.set_state(MenuState::ControlSaveWarning);
                }
            }
            (_, C::Back) => {}
        }
    }

    click.propagate(false);
}

fn controls_changed(
    mut commands: Commands,
    style: Res<Style>,
    controls: Res<ControlsWIP>,
    button: Query<(Entity, &ControlsButtonAction, &Children)>,
) {
    for (entity, action, children) in button.iter() {
        use ControlsButtonAction as C;
        if let C::Prompt(control, idx) = action {
            let key = controls.0.get_control_part(*control, *idx);
            for child in children {
                if let Ok(mut child) = commands.get_entity(*child) {
                    child.despawn();
                }
            }
            commands
                .get_entity(entity)
                .expect("It was just clicked, it should be alive?")
                .remove_children(children)
                .with_children(|builder| input_to_screen(&style, builder, &key));
        }
    }
}

fn remove_control_prompt_target(mut commands: Commands) {
    commands.remove_resource::<ControlPromptTarget>();
}

fn control_prompt_enter(mut commands: Commands, style: Res<Style>) {
    let button_text_style = (
        style.font(33.0),
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
                    style.font(33.0),
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),
                (
                    Text::new("or click 'Cancel'"),
                    style.font(33.0),
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
    mut controls: ResMut<ControlsWIP>,
    cancel_button_query: Query<(), With<ControlsButtonAction>>,
    target: Res<ControlPromptTarget>,
    hover_map: Res<HoverMap>,
) {
    for ev in keyboard.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Keyboard(ev.key_code)));
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

                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Mouse(ev.button)));
                commands.set_state(MenuState::Controls);
                return;
            }
            ButtonState::Released => {}
        }
    }

    for ev in gamepad.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Gamepad(ev.button)));
                commands.set_state(MenuState::Controls);
                return;
            }
            ButtonState::Released => {}
        }
    }
}

fn control_save_warning_enter(mut commands: Commands, style: Res<Style>) {
    let button_text_style = (
        style.font(33.0),
        TextColor(TEXT_COLOR),
        TextLayout::new_with_justify(JustifyText::Center),
    );

    commands
        .spawn((
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
            OnControlSaveWarning,
            BackgroundColor(BACKGROUND_COLOR_SOLID),
            ZIndex(2),
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    display: Display::Flex,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn((
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
                            ControlsButtonAction::Save,
                            MenuButtonAction::Settings,
                            children![(
                                Text::new("Save Changes"),
                                button_text_style.clone(),
                                ControlsButtonAction::PromptCancel
                            )],
                        ))
                        .observe(controls_menu_click)
                        .observe(menu_button_click);
                    builder
                        .spawn((
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
                            ControlsButtonAction::Discard,
                            MenuButtonAction::Settings,
                            children![(
                                Text::new("Discard Changes"),
                                button_text_style.clone(),
                                ControlsButtonAction::PromptCancel
                            )],
                        ))
                        .observe(controls_menu_click)
                        .observe(menu_button_click);
                });
        });
}

/// Helper method to despawn all of the entities with a given component.
/// This is used with the `On*` Components to easily destroy all of the components
/// on specific screens
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

fn remove_resource<T: Resource>(mut commands: Commands) {
    commands.remove_resource::<T>();
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
