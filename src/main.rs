use bevy::{
    prelude::*,
    time::FixedTimestep,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

const GRAVITY: f32 = 100.0;
const FLAPPY_JUMP_STRENGTH: f32 = 1500.0;

const FLAPPY_STARTING_POSITION: Vec3 = Vec3::ZERO;
const FLAPPY_SIZE: Vec3 = Vec3::new(40.0, 20.0, 20.0);

const FLAPPY_COLOUR: Color = Color::rgb(0.3, 0.3, 0.7);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP.into()))
                .with_system(flappy_gravity)
                .with_system(flappy_jump.after(flappy_gravity))
                .with_system(apply_flappy_velocity.after(flappy_jump)),
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

    // Flappy
    commands
        .spawn()
        .insert(Flappy)
        .insert(Velocity(Vec2::ZERO))
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

    flappy_velocity.y -= GRAVITY;
}

fn flappy_jump(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Flappy>>) {
    let mut flappy_velocity = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy_velocity.y = FLAPPY_JUMP_STRENGTH;
    }
}

fn apply_flappy_velocity(mut query: Query<(&mut Transform, &Velocity), With<Flappy>>) {
    let (mut transform, velocity) = query.single_mut();

    transform.translation.y = transform.translation.y + velocity.y * TIME_STEP;
}
