use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::Anchor,
    time::FixedTimestep,
};
use rand::Rng;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

const WINDOW_WIDTH: f32 = 500.0;
const WINDOW_HEIGHT: f32 = 700.0;
const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const GRAVITY: f32 = 40.0;
const SCROLLING_SPEED: f32 = 150.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec2::ZERO.extend(1.0);
const FLAPPY_STARTING_VELOCITY: Vec2 = Vec2::new(SCROLLING_SPEED, 0.0);
const FLAPPY_SIZE: Vec3 = Vec3::new(40.0, 20.0, 0.0);
const FLAPPY_JUMP_STRENGTH: f32 = 700.0;
const FLAPPY_FALL_ROTATION_SPEED: f32 = -4.0;
const FLAPPY_FALL_ROTATION_ANGLE_LIMIT: f32 = 5.0;
const FLAPPY_COLOUR: Color = Color::rgb(0.3, 0.3, 0.7);

// for infinite floor, 3 floor entities reused when one move out of the window
const FLOOR_ENTITY_COUNT: u32 = 3;
const FLOOR_WIDTH: f32 = WINDOW_WIDTH;
const FLOOR_THICKNESS: f32 = 30.0;
const FLOOR_POSITION_Y: f32 = -WINDOW_HEIGHT / 2.0 + FLOOR_THICKNESS;
const FLOOR_STARTING_POSITION_X: f32 = -WINDOW_WIDTH / 2.0;

const PIPE_SET_ENTITY_COUNT: u32 = 3;
const PIPE_GAP: f32 = 200.0;
const PIPE_WIDTH: f32 = 125.0;
const PIPE_DISTANCE: f32 = 350.0;
const DISTANCE_TO_FIRST_PIPE: f32 = 500.0;
const PIPE_GAP_MIN_Y: f32 = -200.0;
const PIPE_GAP_MAX_Y: f32 = 200.0;
const PIPE_COLOR: Color = Color::rgb(0.6, 0.85, 0.4);

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
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP.into()))
                .with_system(flappy_gravity)
                .with_system(flappy_jump)
                .with_system(flappy_apply_velocity)
                .with_system(camera_side_scroll)
                .with_system(floor_side_scroll)
                .with_system(pipe_side_scroll)
                .with_system(check_for_collusion),
        )
        .run();
}

//
// -- COMPONENT
//

#[derive(Component)]
struct Flappy;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Pipe;

enum PipePosition {
    Top,
    Bottom,
}

impl Pipe {
    fn construct_sprite_bundle(translation: Vec3, scale: Vec3, anchor: Anchor) -> SpriteBundle {
        SpriteBundle {
            transform: Transform {
                translation,
                scale,
                ..default()
            },
            sprite: Sprite {
                color: PIPE_COLOR,
                anchor,
                ..default()
            },
            ..default()
        }
    }

    fn sprite_bundle(position: PipePosition, gap_center: &Vec2) -> SpriteBundle {
        match position {
            PipePosition::Top => {
                let pipe_bottom_y = gap_center.y + PIPE_GAP / 2.0;
                let window_top = WINDOW_HEIGHT / 2.0;
                let height_to_top = window_top - pipe_bottom_y;
                let pipe_height = height_to_top + 300.0;

                Self::construct_sprite_bundle(
                    Vec3::new(gap_center.x, pipe_bottom_y, 0.0),
                    Vec3::new(PIPE_WIDTH, pipe_height, 0.0),
                    Anchor::BottomCenter,
                )
            }

            PipePosition::Bottom => {
                let pipe_top_y = gap_center.y - PIPE_GAP / 2.0;
                let window_bottom = -WINDOW_HEIGHT / 2.0;
                let height_to_bottom = pipe_top_y - window_bottom;
                let pipe_height = height_to_bottom + 300.0;

                Self::construct_sprite_bundle(
                    Vec3::new(gap_center.x, pipe_top_y, 0.0),
                    Vec3::new(PIPE_WIDTH, pipe_height, 0.0),
                    Anchor::TopCenter,
                )
            }
        }
    }
}

#[derive(Bundle)]
struct PipeBundle {
    #[bundle]
    sprite: SpriteBundle,
    collider: Collider,
    pipe: Pipe,
}

impl PipeBundle {
    fn new_set(gap_center: &Vec2) -> (Self, Self) {
        (
            PipeBundle {
                sprite: Pipe::sprite_bundle(PipePosition::Top, gap_center),
                collider: Collider,
                pipe: Pipe,
            },
            PipeBundle {
                sprite: Pipe::sprite_bundle(PipePosition::Bottom, gap_center),
                collider: Collider,
                pipe: Pipe,
            },
        )
    }

    fn spawn_set(commands: &mut Commands, position_x: f32) {
        let gap_position = Vec2::new(
            position_x,
            rand::thread_rng().gen_range(PIPE_GAP_MIN_Y..=PIPE_GAP_MAX_Y),
        );

        let (top_pipe, bottom_pipe) = Self::new_set(&gap_position);
        commands.spawn_bundle(top_pipe);
        commands.spawn_bundle(bottom_pipe);
    }
}

#[derive(Component)]
struct Floor;

#[derive(Bundle)]
struct FloorBundle {
    #[bundle]
    sprite: SpriteBundle,
    collider: Collider,
    floor: Floor,
}

impl FloorBundle {
    fn new(index: u32) -> Self {
        let pos = index as f32;
        let translation_x = FLOOR_STARTING_POSITION_X + (pos * FLOOR_WIDTH);

        FloorBundle {
            sprite: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(translation_x, FLOOR_POSITION_Y, 0.0),
                    scale: Vec3::new(FLOOR_WIDTH, FLOOR_THICKNESS, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5 + (pos / 10.0), 0.7),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            floor: Floor,
        }
    }
}

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

fn flappy_gravity(mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    // gravity
    flappy_velocity.y -= GRAVITY;
}

fn flappy_jump(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Flappy>>,
) {
    let (mut flappy_velocity, mut flappy_transform) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy_velocity.y = FLAPPY_JUMP_STRENGTH;
        flappy_transform.rotation = Quat::from_rotation_z(0.5);
    }
}

fn flappy_apply_velocity(mut query: Query<(&mut Transform, &Velocity), With<Flappy>>) {
    let (mut flappy_transform, flappy_velocity) = query.single_mut();

    flappy_transform.translation.x = flappy_transform.translation.x + flappy_velocity.x * TIME_STEP;
    flappy_transform.translation.y = flappy_transform.translation.y + flappy_velocity.y * TIME_STEP;

    // Falling "animation"
    // flappy slowly angled down as it falls, but cap it angle limit
    let quat_limit = Quat::from_rotation_z(FLAPPY_FALL_ROTATION_ANGLE_LIMIT);
    let angle_to_limit = flappy_transform.rotation.angle_between(quat_limit);
    let is_flappy_falling = flappy_velocity.y < 0.0;
    let is_flappy_rotation_close_to_limit = angle_to_limit > 0.2;

    if is_flappy_falling && is_flappy_rotation_close_to_limit {
        flappy_transform.rotate_z(FLAPPY_FALL_ROTATION_SPEED * TIME_STEP);
    }
}

fn camera_side_scroll(mut query: Query<&mut Transform, With<Camera2d>>) {
    let mut camera_transform = query.single_mut();

    camera_transform.translation.x += SCROLLING_SPEED * TIME_STEP;
}

fn floor_side_scroll(
    camera_query: Query<&Transform, With<Camera2d>>,
    mut floor_query: Query<&mut Transform, (With<Floor>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

    // when a floor moved out of sight, reuse it by moving it to the back
    for mut floor_transform in &mut floor_query {
        let floor_right_edge_position = floor_transform.translation.x + FLOOR_WIDTH;
        let camera_left_edge_position = camera_transform.translation.x - (WINDOW_WIDTH / 2.0);
        let buffer = WINDOW_WIDTH / 2.0;
        if floor_right_edge_position + buffer < camera_left_edge_position {
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
        let buffer = WINDOW_WIDTH / 2.0;
        if pipe_right_edge_position + buffer < camera_left_edge_position {
            pipes_to_remove.push((pipe_entity, pipe_transform));
        }
    }

    // if there is pipes that is out of sight, spawn a new set
    // we need 2 pipes as we should remove a set to spawn a new set
    if pipes_to_remove.len() == 2 {
        let last_pipe_position_x = pipes_to_remove[0].1.translation.x;

        // 2 pipes should be in a same set
        assert!(pipes_to_remove[0].1.translation.x == pipes_to_remove[1].1.translation.x);

        for (pipe_entity, _) in pipes_to_remove {
            commands.entity(pipe_entity).despawn();
        }

        let gap_position_x = last_pipe_position_x + PIPE_DISTANCE * (PIPE_SET_ENTITY_COUNT as f32);
        PipeBundle::spawn_set(&mut commands, gap_position_x);
    }
}

fn check_for_collusion(
    flappy_query: Query<&Transform, With<Flappy>>,
    collider_query: Query<&Transform, With<Collider>>,
) {
    let flappy_transform = flappy_query.single();

    // without rotation
    let flappy_left_x = flappy_transform.translation.x - (flappy_transform.scale.x / 2.0);
    let flappy_right_x = flappy_transform.translation.x + (flappy_transform.scale.x / 2.0);
    let flappy_top_y = flappy_transform.translation.y + (flappy_transform.scale.y / 2.0);
    let flappy_bottom_y = flappy_transform.translation.y - (flappy_transform.scale.y / 2.0);

    let flappy_top_left = Vec2::new(flappy_left_x, flappy_top_y);
    let flappy_top_right = Vec2::new(flappy_right_x, flappy_top_y);
    let flappy_bottom_left = Vec2::new(flappy_left_x, flappy_bottom_y);
    let flappy_bottom_right = Vec2::new(flappy_right_x, flappy_bottom_y);

    // collision box with rotation
    let (axis, angle) = flappy_transform.rotation.to_axis_angle();
    let _flappy_collision_vertices = vec![
        point_rotate_around_axis(&flappy_top_left, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_top_right, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_bottom_right, &axis.truncate(), angle),
        point_rotate_around_axis(&flappy_bottom_left, &axis.truncate(), angle),
    ];

    for _collider_transform in &collider_query {
        // TODO do actual collision check
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
