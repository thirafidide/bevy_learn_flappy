use bevy::prelude::*;
use sepax2d::polygon::Polygon;

use crate::velocity::Velocity;
use crate::window::*;

const FLAPPY_SPRITE_SIZE: f32 = 24.0;
const FLAPPY_SPRITE_SCALE: Vec3 = Vec3::splat(2.0);
const FLAPPY_SIZE: Vec3 = Vec3::new(
    FLAPPY_SPRITE_SCALE.x * FLAPPY_SPRITE_SIZE,
    FLAPPY_SPRITE_SCALE.y * FLAPPY_SPRITE_SIZE,
    0.0,
);
const FLAPPY_COLLISION_SIZE: Vec3 = Vec3::new(FLAPPY_SIZE.x * 0.65, FLAPPY_SIZE.y * 0.65, 0.0);
const FLAPPY_JUMP_STRENGTH: f32 = 700.0;
const FLAPPY_FALL_ROTATION_SPEED: f32 = -4.0;
const FLAPPY_FALL_ROTATION_ANGLE_LIMIT: f32 = 5.0;
// Max height flappy can jump above the window height
const FLAPPY_MAX_FLY_HEIGHT: f32 = (WINDOW_HEIGHT / 2.0) + WINDOW_BOUND_LIMIT;
const FLAPPY_JUMP_ANGLE: f32 = 0.5;

#[derive(Component)]
pub struct Flappy;

pub fn spawn(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_map: ResMut<Assets<TextureAtlas>>,
    position: Vec3,
    velocity: Vec2,
) {
    let texture_handle = asset_server.load("characters.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::splat(FLAPPY_SPRITE_SIZE), 9, 3);
    let texture_atlas_handle = texture_atlas_map.add(texture_atlas);

    commands
        .spawn()
        .insert(Flappy)
        .insert(Velocity(velocity))
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                translation: position,
                scale: FLAPPY_SPRITE_SCALE,
                ..default()
            },
            sprite: TextureAtlasSprite {
                index: 24,
                flip_x: true,
                ..default()
            },
            ..default()
        });
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
    let flappy_left_x = transform.translation.x - (FLAPPY_COLLISION_SIZE.x / 2.0);
    let flappy_right_x = transform.translation.x + (FLAPPY_COLLISION_SIZE.x / 2.0);
    let flappy_top_y = transform.translation.y + (FLAPPY_COLLISION_SIZE.y / 2.0);
    let flappy_bottom_y = transform.translation.y - (FLAPPY_COLLISION_SIZE.y / 2.0);

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
