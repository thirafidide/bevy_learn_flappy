use bevy::{prelude::*, time::FixedTimestep};
use sepax2d::prelude::{sat_overlap, Polygon, AABB};

mod collider;
mod floor;
mod pipe;
mod window;

use crate::collider::Collider;
use crate::floor::{Floor, FloorBundle, FLOOR_WIDTH};
use crate::pipe::{Pipe, PipeBundle, PIPE_WIDTH};
use crate::window::*;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;
const GRAVITY: f32 = 40.0;
const SCROLLING_SPEED: f32 = 150.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec2::ZERO.extend(1.0);
const FLAPPY_STARTING_VELOCITY: Vec2 = Vec2::new(SCROLLING_SPEED, 0.0);
const FLAPPY_SIZE: Vec3 = Vec3::new(40.0, 20.0, 0.0);
const FLAPPY_JUMP_STRENGTH: f32 = 700.0;
const FLAPPY_FALL_ROTATION_SPEED: f32 = -4.0;
const FLAPPY_FALL_ROTATION_ANGLE_LIMIT: f32 = 5.0;
const FLAPPY_COLOUR: Color = Color::rgb(0.3, 0.3, 0.7);
// Max height flappy can jump above the window height
const FLAPPY_MAX_FLY_HEIGHT: f32 = (WINDOW_HEIGHT / 2.0) + WINDOW_BOUND_LIMIT;
const FLAPPY_JUMP_ANGLE: f32 = 0.5;

// for infinite floor, 3 floor entities reused when one move out of the window
const FLOOR_ENTITY_COUNT: u32 = 3;

const PIPE_SET_ENTITY_COUNT: u32 = 3;
const PIPE_DISTANCE: f32 = 350.0;
const DISTANCE_TO_FIRST_PIPE: f32 = 500.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: false,
            ..default()
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_state(RunState::Playing)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP.into()))
                .with_system(check_for_collision)
                .with_system(flappy_gravity.before(check_for_collision))
                .with_system(flappy_jump.before(check_for_collision))
                .with_system(flappy_apply_velocity.before(check_for_collision))
                .with_system(camera_side_scroll.before(check_for_collision))
                .with_system(floor_side_scroll.before(check_for_collision))
                .with_system(pipe_side_scroll.before(check_for_collision)),
        )
        .run();
}

//
// -- STATE
//

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum RunState {
    Playing,
    GameOver,
}

//
// -- COMPONENT
//

#[derive(Component)]
struct Flappy;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

#[derive(Default)]
struct CollisionEvent;

//
// -- SETUP
//

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Flappy
    commands
        .spawn()
        .insert(Flappy)
        .insert(Velocity(FLAPPY_STARTING_VELOCITY))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: FLAPPY_STARTING_POSITION,
                scale: FLAPPY_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: FLAPPY_COLOUR,
                ..default()
            },
            ..default()
        });

    // Floor
    for i in 0..FLOOR_ENTITY_COUNT {
        commands.spawn_bundle(FloorBundle::new(i));
    }

    // Pipe
    for i in 0..PIPE_SET_ENTITY_COUNT {
        let gap_position_x = DISTANCE_TO_FIRST_PIPE + (PIPE_DISTANCE * (i as f32));
        PipeBundle::spawn_set(&mut commands, gap_position_x)
    }
}

//
// -- SYSTEM
//

fn flappy_gravity(run_state: Res<State<RunState>>, mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    // gravity
    flappy_velocity.y -= GRAVITY;
}

fn flappy_jump(
    keyboard_input: Res<Input<KeyCode>>,
    run_state: Res<State<RunState>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Flappy>>,
) {
    let (mut flappy_velocity, mut flappy_transform) = query.single_mut();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy_velocity.y = FLAPPY_JUMP_STRENGTH;
        flappy_transform.rotation = Quat::from_rotation_z(FLAPPY_JUMP_ANGLE);
    }
}

fn flappy_apply_velocity(
    run_state: Res<State<RunState>>,
    mut query: Query<(&mut Transform, &Velocity), With<Flappy>>,
) {
    let (mut flappy_transform, flappy_velocity) = query.single_mut();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    flappy_transform.translation.x = flappy_transform.translation.x + flappy_velocity.x * TIME_STEP;
    flappy_transform.translation.y = flappy_transform.translation.y + flappy_velocity.y * TIME_STEP;
    if flappy_transform.translation.y > FLAPPY_MAX_FLY_HEIGHT {
        flappy_transform.translation.y = FLAPPY_MAX_FLY_HEIGHT;
    }

    // Falling "animation"
    // flappy slowly angled down as it falls, but cap it to angle limit
    let quat_limit = Quat::from_rotation_z(FLAPPY_FALL_ROTATION_ANGLE_LIMIT);
    let angle_to_limit = flappy_transform.rotation.angle_between(quat_limit);
    let is_flappy_falling = flappy_velocity.y < 0.0;
    let is_flappy_rotation_close_to_limit = angle_to_limit < 0.2;

    if is_flappy_falling && !is_flappy_rotation_close_to_limit {
        flappy_transform.rotate_z(FLAPPY_FALL_ROTATION_SPEED * TIME_STEP);
    }
}

fn camera_side_scroll(
    run_state: Res<State<RunState>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut camera_transform = query.single_mut();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    camera_transform.translation.x += SCROLLING_SPEED * TIME_STEP;
}

fn floor_side_scroll(
    run_state: Res<State<RunState>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut floor_query: Query<&mut Transform, (With<Floor>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    // when a floor moved out of sight, reuse it by moving it to the back
    for mut floor_transform in &mut floor_query {
        let floor_right_edge_position = floor_transform.translation.x + (FLOOR_WIDTH / 2.0);
        let camera_left_edge_position = camera_transform.translation.x - (WINDOW_WIDTH / 2.0);

        if floor_right_edge_position + WINDOW_BOUND_LIMIT < camera_left_edge_position {
            floor_transform.translation.x += FLOOR_WIDTH * (FLOOR_ENTITY_COUNT as f32);
        }
    }
}

fn pipe_side_scroll(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    pipes_query: Query<(Entity, &Transform), (With<Pipe>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

    // when a pipe moved out of sight, mark it to be despawned
    let mut pipes_to_remove: Vec<(Entity, &Transform)> = Vec::new();
    for (pipe_entity, pipe_transform) in &pipes_query {
        let pipe_right_edge_position = pipe_transform.translation.x + (PIPE_WIDTH / 2.0);
        let camera_left_edge_position = camera_transform.translation.x - (WINDOW_WIDTH / 2.0);

        if pipe_right_edge_position + WINDOW_BOUND_LIMIT < camera_left_edge_position {
            pipes_to_remove.push((pipe_entity, pipe_transform));
        }
    }

    // if there is pipes that is out of sight, spawn a new set
    // we need 2 pipes as we should remove a set to spawn a new set
    if pipes_to_remove.len() == 2 {
        let last_pipe_position_x = pipes_to_remove[0].1.translation.x;

        // 2 pipes should be in a same set
        assert!(
            (pipes_to_remove[0].1.translation.x - pipes_to_remove[1].1.translation.x).abs()
                < f32::EPSILON
        );

        for (pipe_entity, _) in pipes_to_remove {
            commands.entity(pipe_entity).despawn();
        }

        let gap_position_x = last_pipe_position_x + PIPE_DISTANCE * (PIPE_SET_ENTITY_COUNT as f32);
        PipeBundle::spawn_set(&mut commands, gap_position_x);
    }
}

fn check_for_collision(
    flappy_query: Query<&Transform, With<Flappy>>,
    collider_query: Query<&Transform, With<Collider>>,
    mut run_state: ResMut<State<RunState>>,
) {
    let flappy_transform = flappy_query.single();

    // flappy without rotation
    let flappy_left_x = flappy_transform.translation.x - (flappy_transform.scale.x / 2.0);
    let flappy_right_x = flappy_transform.translation.x + (flappy_transform.scale.x / 2.0);
    let flappy_top_y = flappy_transform.translation.y + (flappy_transform.scale.y / 2.0);
    let flappy_bottom_y = flappy_transform.translation.y - (flappy_transform.scale.y / 2.0);

    let flappy_top_left = Vec2::new(flappy_left_x, flappy_top_y);
    let flappy_top_right = Vec2::new(flappy_right_x, flappy_top_y);
    let flappy_bottom_left = Vec2::new(flappy_left_x, flappy_bottom_y);
    let flappy_bottom_right = Vec2::new(flappy_right_x, flappy_bottom_y);

    // collision box with rotation
    let (relative_axis, angle) = flappy_transform.rotation.to_axis_angle();
    let axis = flappy_transform.translation + relative_axis;
    let flappy_collision_vertices = vec![
        point_rotate_around_axis(&flappy_top_left, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_top_right, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_bottom_right, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_bottom_left, &axis.truncate(), angle),
    ]
    .into_iter()
    .map(|point| (point.x, point.y))
    .collect();

    let flappy_collision_polygon = Polygon::from_vertices((0.0, 0.0), flappy_collision_vertices);

    for collider_transform in &collider_query {
        let collider_top_left = (
            collider_transform.translation.x - (collider_transform.scale.x / 2.0),
            collider_transform.translation.y + (collider_transform.scale.y / 2.0),
        );
        let collider_shape = AABB::new(
            collider_top_left,
            collider_transform.scale.x,
            // Doesn't work if the height is positive
            // Either Sepax2D AABB actually use bottom left as position
            // or I'm just bad at math graph
            -collider_transform.scale.y,
        );

        let is_collide = sat_overlap(&flappy_collision_polygon, &collider_shape);

        if *run_state.current() != RunState::GameOver && is_collide {
            match run_state.set(RunState::GameOver) {
                Err(message) => println!("{message:?}"),
                _ => (),
            }
        }
    }
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
