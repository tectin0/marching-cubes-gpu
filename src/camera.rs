use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::CameraMarker;

pub fn camera_control(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&CameraMarker, &mut Transform)>,
) {
    let time_delta = time.delta_seconds();

    let (_, mut transform) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        *transform = Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    }

    let is_shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft);
    let is_middle_mouse_button_pressed = mouse_buttons.pressed(MouseButton::Middle);

    let speed = 1.0 * time_delta;

    let mut delta = Vec2::default();

    for ev in mouse_motion.read() {
        delta += ev.delta * speed;
    }

    if is_middle_mouse_button_pressed {
        if is_shift_pressed {
            let transformed_x = transform.local_x() * -delta.x;
            let transformed_y = transform.local_y() * delta.y;

            transform.translation += transformed_x + transformed_y;
        } else {
            let local_x = transform.local_x();
            let local_y = transform.local_y();

            let yaw = Quat::from_axis_angle(*local_y, -delta.x);
            let pitch = Quat::from_axis_angle(*local_x, -delta.y);

            transform.rotate_around(Vec3::ZERO, yaw);
            transform.rotate_around(Vec3::ZERO, pitch);
        }
    }

    let mut delta = 0.0;
    let speed = 100.0 * time_delta;

    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => delta += ev.y * speed,
            _ => {}
        }
    }

    let transformed = transform.local_z() * delta;
    transform.translation -= transformed;
}
