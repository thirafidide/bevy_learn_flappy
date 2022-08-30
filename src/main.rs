use bevy::{prelude::*, time::FixedTimestep};
use sepax2d::prelude::{sat_overlap, AABB};

mod collider;
mod flappy;
mod floor;
mod pipe;
mod velocity;
mod window;

use crate::collider::Collider;
use crate::flappy::{Flappy, FlappyBundle};
use crate::floor::{Floor, FloorBundle, FLOOR_WIDTH};
use crate::pipe::{Pipe, PipeBundle, PIPE_WIDTH};
use crate::velocity::Velocity;
use crate::window::*;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;
const GRAVITY: f32 = 40.0;
const SCROLLING_SPEED: f32 = 150.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec2::ZERO.extend(1.0);
const FLAPPY_STARTING_VELOCITY: Vec2 = Vec2::new(SCROLLING_SPEED, 0.0);

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
// -- SETUP
//

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Flappy
    commands.spawn_bundle(FlappyBundle::new(
        FLAPPY_STARTING_POSITION,
        FLAPPY_STARTING_VELOCITY,
    ));

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
    let (flappy_velocity, flappy_transform) = query.single_mut();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy::jump(flappy_transform, flappy_velocity);
    }
}

fn flappy_apply_velocity(
    run_state: Res<State<RunState>>,
    mut query: Query<(&mut Transform, &Velocity), With<Flappy>>,
) {
    let (flappy_transform, flappy_velocity) = query.single_mut();

    if *run_state.current() == RunState::GameOver {
        return;
    }

    flappy::apply_velocity(flappy_transform, flappy_velocity, TIME_STEP);
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
    let flappy_collision_polygon = flappy::to_polygon(flappy_transform);

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
