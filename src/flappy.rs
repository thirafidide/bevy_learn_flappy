use bevy::prelude::*;
use sepax2d::polygon::Polygon;

use crate::velocity::Velocity;
use crate::window::*;

const FLAPPY_SIZE: Vec3 = Vec3::new(40.0, 20.0, 0.0);
const FLAPPY_JUMP_STRENGTH: f32 = 700.0;
const FLAPPY_FALL_ROTATION_SPEED: f32 = -4.0;
const FLAPPY_FALL_ROTATION_ANGLE_LIMIT: f32 = 5.0;
const FLAPPY_COLOUR: Color = Color::rgb(0.3, 0.3, 0.7);
// Max height flappy can jump above the window height
const FLAPPY_MAX_FLY_HEIGHT: f32 = (WINDOW_HEIGHT / 2.0) + WINDOW_BOUND_LIMIT;
const FLAPPY_JUMP_ANGLE: f32 = 0.5;

#[derive(Component)]
pub struct Flappy;

#[derive(Bundle)]
pub struct FlappyBundle {
    flappy: Flappy,
    velocity: Velocity,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl FlappyBundle {
    pub fn new(position: Vec3, velocity: Vec2) -> FlappyBundle {
        FlappyBundle {
            flappy: Flappy,
            velocity: Velocity(velocity),
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    scale: FLAPPY_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: FLAPPY_COLOUR,
                    ..default()
                },
                ..default()
            },
        }
    }
}

//
// -- System
//

pub fn jump(mut transform: Mut<Transform>, mut velocity: Mut<Velocity>) {
    velocity.y = FLAPPY_JUMP_STRENGTH;
    transform.rotation = Quat::from_rotation_z(FLAPPY_JUMP_ANGLE);
}

pub fn apply_velocity(mut transform: Mut<Transform>, velocity: &Velocity, delta: f32) {
    transform.translation.x = transform.translation.x + velocity.x * delta;
    transform.translation.y = transform.translation.y + velocity.y * delta;
    if transform.translation.y > FLAPPY_MAX_FLY_HEIGHT {
        transform.translation.y = FLAPPY_MAX_FLY_HEIGHT;
    }

    // Falling "animation"
    // flappy slowly angled down as it falls, but cap it to angle limit
    let quat_limit = Quat::from_rotation_z(FLAPPY_FALL_ROTATION_ANGLE_LIMIT);
    let angle_to_limit = transform.rotation.angle_between(quat_limit);
    let is_falling = velocity.y < 0.0;
    let is_rotation_close_to_limit = angle_to_limit < 0.2;

    if is_falling && !is_rotation_close_to_limit {
        transform.rotate_z(FLAPPY_FALL_ROTATION_SPEED * delta);
    }
}

pub fn to_polygon(transform: &Transform) -> Polygon {
    // flappy without rotation
    let flappy_left_x = transform.translation.x - (transform.scale.x / 2.0);
    let flappy_right_x = transform.translation.x + (transform.scale.x / 2.0);
    let flappy_top_y = transform.translation.y + (transform.scale.y / 2.0);
    let flappy_bottom_y = transform.translation.y - (transform.scale.y / 2.0);

    let flappy_top_left = Vec2::new(flappy_left_x, flappy_top_y);
    let flappy_top_right = Vec2::new(flappy_right_x, flappy_top_y);
    let flappy_bottom_left = Vec2::new(flappy_left_x, flappy_bottom_y);
    let flappy_bottom_right = Vec2::new(flappy_right_x, flappy_bottom_y);

    // collision box with rotation
    let (relative_axis, angle) = transform.rotation.to_axis_angle();
    let axis = transform.translation + relative_axis;
    let flappy_collision_vertices = vec![
        point_rotate_around_axis(&flappy_top_left, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_top_right, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_bottom_right, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_bottom_left, &axis.truncate(), angle),
    ]
    .into_iter()
    .map(|point| (point.x, point.y))
    .collect();

    Polygon::from_vertices((0.0, 0.0), flappy_collision_vertices)
}

//
// -- UTILS
//

fn point_rotate(point: &Vec2, angle: f32) -> Vec2 {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();

    Vec2 {
        x: point.x * cos_angle - point.y * sin_angle,
        y: point.y * cos_angle + point.x * sin_angle,
    }
}

fn point_rotate_around_axis(point: &Vec2, axis: &Vec2, angle: f32) -> Vec2 {
    let new_point = Vec2 {
        x: point.x - axis.x,
        y: point.y - axis.y,
    };
    let mut new_point = point_rotate(&new_point, angle);

    new_point.x += axis.x;
    new_point.y += axis.y;

    new_point
}
