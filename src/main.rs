use bevy::prelude::*;
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
        .insert_resource(Scoreboard::new())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_state(RunState::Playing)
        .add_system_set(SystemSet::on_enter(RunState::Playing).with_system(reset_current_score))
        .add_system_set(
            SystemSet::on_update(RunState::Playing)
                .with_system(check_for_collision)
                .with_system(flappy_gravity.before(check_for_collision))
                .with_system(flappy_jump.before(check_for_collision))
                .with_system(flappy_apply_velocity.before(check_for_collision))
                .with_system(camera_side_scroll.before(check_for_collision))
                .with_system(floor_side_scroll.before(check_for_collision))
                .with_system(pipe_side_scroll.before(check_for_collision))
                .with_system(update_current_score.after(check_for_collision)),
        )
        .add_system_set(
            SystemSet::on_enter(RunState::GameOver)
                .with_system(update_best_score)
                .with_system(flappy_forward_stop),
        )
        .add_system_set(
            SystemSet::on_update(RunState::GameOver)
                .with_system(gameover_input)
                .with_system(flappy_gravity)
                .with_system(flappy_apply_velocity),
        )
        .add_system_set(SystemSet::on_enter(RunState::Cleanup).with_system(reset_setup))
        .add_system_set(SystemSet::on_update(RunState::Cleanup).with_system(cleanup_finished))
        .run();
}

//
// -- STATE
//

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum RunState {
    Playing,
    GameOver,
    Cleanup,
}

#[derive(Debug, Clone)]
struct Scoreboard {
    current_score: u32,
    best_score: u32,
}

impl Scoreboard {
    fn new() -> Self {
        Self {
            current_score: 0,
            best_score: 0,
        }
    }
}

//
// -- SETUP
//

fn setup_floor(commands: &mut Commands) {
    for i in 0..FLOOR_ENTITY_COUNT {
        commands.spawn_bundle(FloorBundle::new(i));
    }
}

fn setup_pipes(commands: &mut Commands) {
    for i in 0..PIPE_SET_ENTITY_COUNT {
        let gap_position_x = DISTANCE_TO_FIRST_PIPE + (PIPE_DISTANCE * (i as f32));
        PipeBundle::spawn_set(commands, gap_position_x)
    }
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Flappy
    commands.spawn_bundle(FlappyBundle::new(
        FLAPPY_STARTING_POSITION,
        FLAPPY_STARTING_VELOCITY,
    ));

    setup_floor(&mut commands);
    setup_pipes(&mut commands);
}

fn reset_setup(
    mut commands: Commands,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Flappy>)>,
    mut flappy_query: Query<(&mut Transform, &mut Velocity), With<Flappy>>,
    floor_query: Query<Entity, With<Floor>>,
    pipe_query: Query<Entity, With<Pipe>>,
) {
    let mut camera_transform = camera_query.single_mut();
    let (mut flappy_transform, mut flappy_velocity) = flappy_query.single_mut();

    let default_transform = Camera2dBundle::default().transform;
    camera_transform.translation = default_transform.translation.clone();

    flappy_transform.translation = FLAPPY_STARTING_POSITION;
    flappy_transform.rotation = Quat::default();
    flappy_velocity.0 = FLAPPY_STARTING_VELOCITY;

    for floor_entity in floor_query.iter() {
        commands.entity(floor_entity).despawn();
    }

    for pipe_entity in pipe_query.iter() {
        commands.entity(pipe_entity).despawn();
    }

    setup_floor(&mut commands);
    setup_pipes(&mut commands);
}

//
// -- SYSTEM
//

fn reset_current_score(mut scoreboard: ResMut<Scoreboard>) {
    scoreboard.current_score = 0;
}

fn update_current_score(
    mut scoreboard: ResMut<Scoreboard>,
    flappy_query: Query<&Transform, With<Flappy>>,
) {
    let flappy_transform = flappy_query.single();

    scoreboard.current_score =
        ((flappy_transform.translation.x - DISTANCE_TO_FIRST_PIPE) / PIPE_DISTANCE).round() as u32;
}

fn update_best_score(mut scoreboard: ResMut<Scoreboard>) {
    if scoreboard.current_score > scoreboard.best_score {
        scoreboard.best_score = scoreboard.current_score;
    }
}

fn gameover_input(keyboard_input: Res<Input<KeyCode>>, mut run_state: ResMut<State<RunState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        run_state.set(RunState::Cleanup).unwrap();
    }
}

fn cleanup_finished(mut run_state: ResMut<State<RunState>>) {
    run_state.set(RunState::Playing).unwrap();
}

fn flappy_gravity(mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    // gravity
    flappy_velocity.y -= GRAVITY;
}

fn flappy_jump(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Flappy>>,
) {
    let (flappy_velocity, flappy_transform) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy::jump(flappy_transform, flappy_velocity);
    }
}

fn flappy_apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<Flappy>>,
) {
    let (flappy_transform, flappy_velocity) = query.single_mut();

    flappy::apply_velocity(flappy_transform, flappy_velocity, time.delta_seconds());
}

fn flappy_forward_stop(mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    flappy_velocity.x = 0.0;
}

fn camera_side_scroll(time: Res<Time>, mut query: Query<&mut Transform, With<Camera2d>>) {
    let mut camera_transform = query.single_mut();

    camera_transform.translation.x += SCROLLING_SPEED * time.delta_seconds();
}

fn floor_side_scroll(
    camera_query: Query<&Transform, With<Camera2d>>,
    mut floor_query: Query<&mut Transform, (With<Floor>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

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

        if is_collide {
            run_state.set(RunState::GameOver).unwrap();
        }
    }
}
