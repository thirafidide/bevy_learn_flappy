use animation::AnimationReplayEvent;
use bevy::{prelude::*, render::texture::ImageSettings};
use velocity::{ApplyVelocitySystem, VelocityPlugin};

mod animation;
mod collider;
mod flappy;
mod floor;
mod game_state;
mod pipe;
mod velocity;
mod window;

use crate::animation::AnimationPlugin;
use crate::collider::Collider;
use crate::flappy::Flappy;
use crate::floor::{Floor, FloorPlugin};
use crate::game_state::*;
use crate::pipe::{Pipe, PipeBundle, PIPE_WIDTH};
use crate::velocity::Velocity;
use crate::window::*;

const GRAVITY: f32 = 40.0;
const SCROLLING_SPEED: f32 = 150.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec2::ZERO.extend(1.0);
const FLAPPY_STARTING_VELOCITY: Vec2 = Vec2::new(SCROLLING_SPEED, 0.0);
// Max height flappy can jump above the window height
const FLAPPY_MAX_FLY_HEIGHT: f32 = (WINDOW_HEIGHT / 2.0) + WINDOW_BOUND_LIMIT;

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
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(Scoreboard::new())
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin)
        .add_plugin(VelocityPlugin)
        .add_plugin(FloorPlugin)
        .add_startup_system(setup)
        .add_state(GameState::Playing)
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(check_for_collision)
                .with_system(flappy_gravity.before(check_for_collision))
                .with_system(flappy_jump.before(check_for_collision))
                .with_system(flappy_limit_movement.after(ApplyVelocitySystem))
                .with_system(camera_side_scroll.before(check_for_collision))
                .with_system(pipe_side_scroll.before(check_for_collision))
                .with_system(update_current_score.after(check_for_collision)),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::GameOver)
                .with_system(update_best_score)
                .with_system(flappy_forward_stop),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GameOver)
                .with_system(gameover_input)
                .with_system(flappy_gravity),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Cleanup)
                .with_system(reset_current_score)
                .with_system(reset_setup),
        )
        .add_system_set(SystemSet::on_update(GameState::Cleanup).with_system(cleanup_finished))
        .run();
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

fn setup_pipes(commands: &mut Commands) {
    for i in 0..PIPE_SET_ENTITY_COUNT {
        let gap_position_x = DISTANCE_TO_FIRST_PIPE + (PIPE_DISTANCE * (i as f32));
        PipeBundle::spawn_set(commands, gap_position_x)
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_map: ResMut<Assets<TextureAtlas>>,
) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Flappy
    flappy::spawn(
        &mut commands,
        asset_server,
        texture_atlas_map,
        FLAPPY_STARTING_POSITION,
        FLAPPY_STARTING_VELOCITY,
    );

    floor::setup(&mut commands);
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

    floor::setup(&mut commands);
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

fn gameover_input(keyboard_input: Res<Input<KeyCode>>, mut run_state: ResMut<State<GameState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        _ = run_state.set(GameState::Cleanup);
    }
}

fn cleanup_finished(mut run_state: ResMut<State<GameState>>) {
    _ = run_state.set(GameState::Playing);
}

fn flappy_gravity(mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    // gravity
    flappy_velocity.y -= GRAVITY;
}

fn flappy_jump(
    mut replay_event: EventWriter<AnimationReplayEvent>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity), With<Flappy>>,
) {
    let (flappy_entity, flappy_velocity) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy::jump(flappy_velocity);
        replay_event.send(AnimationReplayEvent(flappy_entity));
    }
}

fn flappy_limit_movement(mut query: Query<&mut Transform, With<Flappy>>) {
    let mut flappy_transform = query.single_mut();

    if flappy_transform.translation.y > FLAPPY_MAX_FLY_HEIGHT {
        flappy_transform.translation.y = FLAPPY_MAX_FLY_HEIGHT;
    }
}

fn flappy_forward_stop(mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    flappy_velocity.x = 0.0;
}

fn camera_side_scroll(time: Res<Time>, mut query: Query<&mut Transform, With<Camera2d>>) {
    let mut camera_transform = query.single_mut();

    camera_transform.translation.x += SCROLLING_SPEED * time.delta_seconds();
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
    mut run_state: ResMut<State<GameState>>,
) {
    let flappy_transform = flappy_query.single();

    for collider_transform in &collider_query {
        let collision = flappy::collide(
            &flappy_transform,
            collider_transform.translation,
            collider_transform.scale.truncate(),
        );

        if collision.is_some() {
            run_state.set(GameState::GameOver).unwrap();
        }
    }
}
