use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_inspector_egui::WorldInspectorPlugin;
use flappy::{FlappyCollider, FlappyPlugin};
use gravity::GravityPlugin;
use pipe::PipeSet;
use score::ScorePlugin;
use velocity::VelocityPlugin;

mod animation;
mod collider;
mod flappy;
mod floor;
mod game_state;
mod gravity;
mod pipe;
mod score;
mod velocity;
mod window;

use crate::animation::AnimationPlugin;
use crate::flappy::Flappy;
use crate::floor::{Floor, FloorPlugin};
use crate::game_state::*;
use crate::pipe::{PipeBundle, PIPE_WIDTH};
use crate::velocity::Velocity;
use crate::window::*;

const SCROLLING_SPEED: f32 = 150.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec2::ZERO.extend(1.0);
const FLAPPY_STARTING_VELOCITY: Vec2 = Vec2::new(SCROLLING_SPEED, 0.0);

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
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(AnimationPlugin)
        .add_plugin(VelocityPlugin)
        .add_plugin(GravityPlugin)
        .add_plugin(FloorPlugin)
        .add_plugin(FlappyPlugin)
        .add_plugin(ScorePlugin)
        .add_startup_system(setup)
        .add_state(GameState::Playing)
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(camera_side_scroll)
                .with_system(pipe_side_scroll),
        )
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(gameover_input))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(reset_setup))
        .run();
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
    mut flappy_query: Query<(&mut Transform, &mut Velocity, &mut FlappyCollider), With<Flappy>>,
    floor_query: Query<Entity, With<Floor>>,
    pipe_set_query: Query<Entity, With<PipeSet>>,
) {
    for floor_entity in floor_query.iter() {
        commands.entity(floor_entity).despawn();
    }

    for pipe_set_entity in pipe_set_query.iter() {
        commands.entity(pipe_set_entity).despawn_recursive();
    }

    let mut camera_transform = camera_query.single_mut();
    let (mut flappy_transform, mut flappy_velocity, mut flappy_collider) =
        flappy_query.single_mut();

    let default_transform = Camera2dBundle::default().transform;
    camera_transform.translation = default_transform.translation.clone();

    flappy_transform.translation = FLAPPY_STARTING_POSITION;
    flappy_transform.rotation = Quat::default();
    flappy_velocity.0 = FLAPPY_STARTING_VELOCITY;

    floor::setup(&mut commands);
    setup_pipes(&mut commands);

    flappy_collider.enabled = true;
}

//
// -- SYSTEM
//

fn gameover_input(keyboard_input: Res<Input<KeyCode>>, mut run_state: ResMut<State<GameState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        _ = run_state.set(GameState::Playing);
    }
}

fn camera_side_scroll(time: Res<Time>, mut query: Query<&mut Transform, With<Camera2d>>) {
    let mut camera_transform = query.single_mut();

    camera_transform.translation.x += SCROLLING_SPEED * time.delta_seconds();
}

fn pipe_side_scroll(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    pipe_sets_query: Query<(Entity, &Transform), (With<PipeSet>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

    // when a pipe moved out of sight, despawn it and spawn a new one at the back
    for (pipe_sets_entity, pipe_transform) in &pipe_sets_query {
        let pipe_right_edge_position = pipe_transform.translation.x + (PIPE_WIDTH / 2.0);
        let camera_left_edge_position = camera_transform.translation.x - (WINDOW_WIDTH / 2.0);

        if pipe_right_edge_position + WINDOW_BOUND_LIMIT < camera_left_edge_position {
            let new_gap_position_x =
                pipe_transform.translation.x + PIPE_DISTANCE * (PIPE_SET_ENTITY_COUNT as f32);

            PipeBundle::spawn_set(&mut commands, new_gap_position_x);
            commands.entity(pipe_sets_entity).despawn_recursive();
        }
    }
}
