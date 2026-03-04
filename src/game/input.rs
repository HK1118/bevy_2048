use bevy::prelude::*;

use super::board::Direction;

#[derive(Message)]
pub(super) struct Slide(pub(super) Direction);

pub(super) fn on_drag_end(drag_end: On<Pointer<DragEnd>>, mut move_message: MessageWriter<Slide>) {
    if drag_end.button == PointerButton::Primary {
        if drag_end.distance.length() < 50.0 {
            return;
        }

        if drag_end.distance.x.abs() > drag_end.distance.y.abs() {
            if drag_end.distance.x > 0.0 {
                move_message.write(Slide(Direction::Right));
            } else {
                move_message.write(Slide(Direction::Left));
            }
        } else if drag_end.distance.y > 0.0 {
            move_message.write(Slide(Direction::Down));
        } else {
            move_message.write(Slide(Direction::Up));
        }
    }
}

pub(super) fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut move_message: MessageWriter<Slide>,
) {
    if keys.just_pressed(KeyCode::KeyW)
        || keys.just_pressed(KeyCode::ArrowUp)
        || keys.just_pressed(KeyCode::KeyK)
    {
        move_message.write(Slide(Direction::Up));
    } else if keys.just_pressed(KeyCode::KeyS)
        || keys.just_pressed(KeyCode::ArrowDown)
        || keys.just_pressed(KeyCode::KeyJ)
    {
        move_message.write(Slide(Direction::Down));
    } else if keys.just_pressed(KeyCode::KeyA)
        || keys.just_pressed(KeyCode::ArrowLeft)
        || keys.just_pressed(KeyCode::KeyH)
    {
        move_message.write(Slide(Direction::Left));
    } else if keys.just_pressed(KeyCode::KeyD)
        || keys.just_pressed(KeyCode::ArrowRight)
        || keys.just_pressed(KeyCode::KeyL)
    {
        move_message.write(Slide(Direction::Right));
    }
}
