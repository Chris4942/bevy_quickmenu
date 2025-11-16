use bevy::{
    input::{gamepad::GamepadAxisChangedEvent, keyboard::KeyboardInput},
    prelude::*,
};

use crate::{
    types::{
        self, ButtonComponent, CleanUpUI, GamepadActivation, MenuAssets, NavigationEvent,
        QuickMenuComponent,
    },
    ActionTrait, MenuState, RedrawEvent, ScreenTrait, Selections,
};

// TODO: make this configurable by consumers
//
const STICK_THRESHOLD: f32 = 0.10;

pub fn keyboard_input_system(
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut writer: MessageWriter<NavigationEvent>,
    mut axis_events: MessageReader<GamepadAxisChangedEvent>,
    gamepads: Query<&Gamepad>,
    mut gamepad_activations: Query<&mut GamepadActivation>,
) {
    use NavigationEvent::*;
    for event in keyboard_input.read() {
        match event.key_code {
            KeyCode::ArrowDown => {
                writer.write(Down);
            }
            KeyCode::ArrowUp => {
                writer.write(Up);
            }
            KeyCode::Enter => {
                writer.write(Select);
            }
            KeyCode::Backspace => {
                writer.write(Back);
            }
            _ => {}
        };
    }

    for gamepad in gamepads {
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            writer.write(Down);
        } else if gamepad.just_pressed(GamepadButton::DPadUp) {
            writer.write(Up);
        } else if gamepad.just_pressed(GamepadButton::DPadRight) {
            writer.write(Back);
        } else if gamepad.just_pressed(GamepadButton::South)
            || gamepad.just_pressed(GamepadButton::West)
        {
            writer.write(Select);
        } else if gamepad.just_pressed(GamepadButton::East)
            || gamepad.just_pressed(GamepadButton::North)
        {
            writer.write(Back);
        }
    }

    for event in axis_events.read() {
        let Ok(mut gamepad_activation) = gamepad_activations.get_mut(event.entity) else {
            continue;
        };
        let current = event.value;
        let previous = gamepad_activation.insert(event.axis, event.value);
        match event.axis {
            GamepadAxis::LeftStickY | GamepadAxis::RightStickY => {
                if cross_threshold(current, previous, STICK_THRESHOLD, true) {
                    writer.write(Up);
                } else if cross_threshold(current, previous, -STICK_THRESHOLD, false) {
                    writer.write(Down);
                }
            }
            GamepadAxis::LeftStickX | GamepadAxis::RightStickX => {
                if cross_threshold(current, previous, -STICK_THRESHOLD, false) {
                    writer.write(Back);
                }
            }
            _ => {}
        }
    }
}

fn cross_threshold(current: f32, previous: f32, v: f32, positive: bool) -> bool {
    if positive {
        current > v && v > previous
    } else {
        current < v && v < previous
    }
}

pub fn insert_gamepad_activation_system(
    gamepads: Query<Entity, (With<Gamepad>, Without<GamepadActivation>)>,
    mut commands: Commands,
) {
    for gamepad in gamepads {
        commands.entity(gamepad).insert(GamepadActivation::new());
    }
}

pub fn redraw_system<S>(
    mut commands: Commands,
    existing: Query<Entity, With<QuickMenuComponent>>,
    mut menu_state: ResMut<MenuState<S>>,
    selections: Res<Selections>,
    redraw_reader: MessageReader<RedrawEvent>,
    assets: Res<MenuAssets>,
    // mut initial_render_done: Local<bool>,
) where
    S: ScreenTrait + 'static,
{
    let mut can_redraw = !redraw_reader.is_empty();
    if !menu_state.initial_render_done {
        menu_state.initial_render_done = true;
        can_redraw = true;
    }
    if can_redraw {
        for item in existing.iter() {
            commands.entity(item).despawn();
        }
        menu_state.menu.show(&assets, &selections, &mut commands);
    }
}

pub fn input_system<S>(
    mut reader: MessageReader<NavigationEvent>,
    mut menu_state: ResMut<MenuState<S>>,
    mut redraw_writer: MessageWriter<RedrawEvent>,
    mut selections: ResMut<Selections>,
    mut event_writer: MessageWriter<<<S as ScreenTrait>::Action as ActionTrait>::Event>,
) where
    S: ScreenTrait + 'static,
{
    if let Some(event) = reader.read().next() {
        if let Some(selection) = menu_state.menu.apply_event(event, &mut selections) {
            menu_state
                .menu
                .handle_selection(&selection, &mut event_writer);
        }
        redraw_writer.write(RedrawEvent);
    }
}

#[allow(clippy::type_complexity)]
pub fn mouse_system<S>(
    mut menu_state: ResMut<MenuState<S>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &types::ButtonComponent<S>,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut event_writer: MessageWriter<<<S as ScreenTrait>::Action as ActionTrait>::Event>,
    mut selections: ResMut<Selections>,
    mut redraw_writer: MessageWriter<RedrawEvent>,
) where
    S: ScreenTrait + 'static,
{
    for (
        interaction,
        ButtonComponent {
            selection,
            style,
            menu_identifier,
            selected,
        },
        mut background_color,
    ) in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                // pop to the chosen selection stack entry
                menu_state.menu.pop_to_selection(selection);

                // pre-select the correct row
                selections
                    .0
                    .insert(menu_identifier.0.clone(), menu_identifier.1);
                if let Some(current) = menu_state
                    .menu
                    .apply_event(&NavigationEvent::Select, &mut selections)
                {
                    menu_state
                        .menu
                        .handle_selection(&current, &mut event_writer);
                    redraw_writer.write(RedrawEvent);
                }
            }
            Interaction::Hovered => {
                if !selected {
                    background_color.0 = style.hover.bg;
                }
            }
            Interaction::None => {
                if !selected {
                    background_color.0 = style.normal.bg;
                }
            }
        }
    }
}

/// If the `CleanUpUI` `Resource` is available, remove the menu and then the resource.
/// This is used to close the menu when it is not needed anymore.
pub fn cleanup_system<S>(
    mut commands: Commands,
    existing: Query<Entity, With<types::QuickMenuComponent>>,
) where
    S: ScreenTrait + 'static,
{
    // Remove all menu elements
    for item in existing.iter() {
        commands.entity(item).despawn();
    }
    // Remove the resource again
    commands.remove_resource::<CleanUpUI>();
    // Remove the state
    commands.remove_resource::<MenuState<S>>();
}
