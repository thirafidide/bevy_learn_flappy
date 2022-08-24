use bevy::{prelude::*, render::camera, time::FixedTimestep};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

const GRAVITY: f32 = 60.0;
const SCROLLING_SPEED: f32 = 100.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec3::ZERO;
const FLAPPY_STARTING_VELOCITY: Vec2 = Vec2::new(SCROLLING_SPEED, 0.0);
const FLAPPY_SIZE: Vec3 = Vec3::new(40.0, 20.0, 20.0);
const FLAPPY_JUMP_STRENGTH: f32 = 1000.0;
const FLAPPY_FALL_ROTATION_SPEED: f32 = -4.0;
const FLAPPY_FALL_ROTATION_ANGLE_LIMIT: f32 = 5.0;

const FLAPPY_COLOUR: Color = Color::rgb(0.3, 0.3, 0.7);
const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP.into()))
                .with_system(flappy_gravity)
                .with_system(flappy_jump.after(flappy_gravity))
                .with_system(flappy_apply_velocity.after(flappy_jump))
                .with_system(camera_side_scroll.after(flappy_apply_velocity)),
        )
        .run();
}

#[derive(Component)]
struct Flappy;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Debug
    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform {
            translation: FLAPPY_STARTING_POSITION,
            scale: Vec3::new(400.0, 200.0, 200.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.5, 0.5, 0.7),
            ..default()
        },
        ..default()
    });

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
