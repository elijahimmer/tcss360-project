use super::*;
use crate::prelude::*;

use bevy::{
    a11y::AccessibilityNode,
    ecs::hierarchy::ChildSpawnerCommands,
    input::{
        ButtonState, gamepad::GamepadButtonChangedEvent, keyboard::KeyboardInput,
        mouse::MouseButtonInput,
    },
    picking::hover::HoverMap,
    prelude::*,
    ui::FocusPolicy,
};

use crate::controls::Control;
use crate::controls::{Input, Keybind, input_to_screen};

pub struct MenuControlsPlugin;

impl Plugin for MenuControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ControlsState>()
            .add_systems(
                OnEnter(MenuState::Controls),
                (controls_enter, init_resource::<ControlsWIP>),
            )
            .add_systems(
                OnExit(MenuState::Controls),
                (
                    set_state(ControlsState::Main),
                    despawn_all_with::<OnControls>,
                    remove_resource::<ControlsWIP>,
                ),
            )
            .add_systems(
                Update,
                (
                    controls_changed.run_if(resource_exists_and_changed::<ControlsWIP>),
                    escape_out,
                )
                    .run_if(in_state(MenuState::Controls)),
            )
            .add_systems(OnEnter(ControlsState::Prompt), control_prompt_enter)
            .add_systems(
                OnExit(ControlsState::Prompt),
                (
                    despawn_all_with::<OnPrompt>,
                    remove_resource::<PromptTarget>,
                ),
            )
            .add_systems(
                Update,
                assign_key_input.run_if(in_state(ControlsState::Prompt)),
            )
            .add_systems(
                OnEnter(ControlsState::SaveWarning),
                control_save_warning_enter,
            )
            .add_systems(
                OnExit(ControlsState::SaveWarning),
                despawn_all_with::<OnSaveWarning>,
            );
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ControlsState {
    #[default]
    Main,
    Prompt,
    SaveWarning,
}

#[derive(Component)]
pub struct OnControls;

#[derive(Component)]
pub struct OnPrompt;

#[derive(Component)]
pub struct OnSaveWarning;

#[derive(Component, Clone, Debug)]
pub enum ControlsButtonAction {
    Prompt(Control, usize),
    PromptCancel,
    ResetBoth(Control),
    ResetAll,
    Save,
    Discard,
    Back,
}

fn escape_out(
    controls_state: Res<State<ControlsState>>,
    mut next_controls_state: ResMut<NextState<ControlsState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    controls_master: Res<Controls>,
    controls_wip: Res<ControlsWIP>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
        use ControlsState as C;
        match *controls_state.get() {
            C::Prompt => {
                // ignore, the prompt handles the input.
            }
            C::SaveWarning => {
                next_menu_state.set(MenuState::Settings);
            }
            C::Main => {
                if controls_wip.0 == *controls_master {
                    next_menu_state.set(MenuState::Settings);
                } else {
                    next_controls_state.set(ControlsState::SaveWarning);
                }
            }
        }
    }
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
                    BackgroundColor(style.background_color),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Button,
                        button_node.clone(),
                        BackgroundColor(style.button_color),
                        ControlsButtonAction::Back,
                        children![(Text::new("Back"), button_text_style.clone(), Pickable::IGNORE)],
                    ))
                        .observe(controls_menu_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            ControlsButtonAction::Save,
                            children![(Text::new("Save"), button_text_style.clone())],
                        ))
                        .observe(controls_menu_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            ControlsButtonAction::Discard,
                            children![(Text::new("Discard"), button_text_style.clone())],
                        ))
                        .observe(controls_menu_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
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
                        TextColor(style.title_color),
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
                        BackgroundColor(style.button_color),
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
                    BackgroundColor(style.button_color),
                    ControlsButtonAction::ResetBoth(control),
                    AccessibilityNode(Accessible::new(Role::ListItem)),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                    children![(
                        Text("Reset Both".into()),
                        style.font(33.0),
                        TextColor(style.text_color)
                    )],
                ))
                .observe(controls_menu_click);
        });
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ControlsWIP(pub Controls);

impl FromWorld for ControlsWIP {
    fn from_world(world: &mut World) -> Self {
        Self(
            world
                .get_resource::<Controls>()
                .expect("There should be controls by now!")
                .clone(),
        )
    }
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
                commands.insert_resource(PromptTarget(*control, *entry));
                commands.set_state(ControlsState::Prompt);
            }
            (P::Secondary, C::Prompt(control, entry)) => {
                controls_wip.0.set_control(*control, *entry, None);
            }
            (P::Middle, C::Prompt(control, entry)) => {
                controls_wip.0.reset_control_part(*control, *entry);
            }
            (P::Primary, C::PromptCancel) => commands.set_state(ControlsState::Main),
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
                    commands.set_state(ControlsState::SaveWarning);
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

fn control_prompt_enter(mut commands: Commands, style: Res<Style>) {
    let button_text_style = (
        style.font(33.0),
        TextColor(style.text_color),
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
        OnPrompt,
        BackgroundColor(style.background_color.with_alpha(1.0)),
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
                    TextColor(style.text_color),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),
                (
                    Text::new("or click 'Cancel'"),
                    style.font(33.0),
                    TextColor(style.text_color),
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
                    BackgroundColor(style.button_color),
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
    target: Res<PromptTarget>,
    hover_map: Res<HoverMap>,
) {
    for ev in keyboard.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Keyboard(ev.key_code)));
                commands.set_state(ControlsState::Main);
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
                                commands.set_state(ControlsState::Main);
                                return;
                            }
                        }
                    }
                }

                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Mouse(ev.button)));
                commands.set_state(ControlsState::Main);
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
                commands.set_state(ControlsState::Main);
                return;
            }
            ButtonState::Released => {}
        }
    }
}

fn control_save_warning_enter(mut commands: Commands, style: Res<Style>) {
    let button_text_style = (
        style.font(33.0),
        TextColor(style.text_color),
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
            OnSaveWarning,
            BackgroundColor(style.background_color.with_alpha(1.0)),
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
                            BackgroundColor(style.button_color),
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
                            BackgroundColor(style.button_color),
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
