use bevy::prelude::*;
use bevy::sprite::collide_aabb;

use crate::animation::{Animation, AnimationReplayEvent};
use crate::collider::Collider;
use crate::game_state::GameState;
use crate::gravity::GravityAffected;
use crate::pipe::PipeGap;
use crate::score::Scoreboard;
use crate::velocity::{ApplyVelocitySystem, Velocity};
use crate::{window::*, SCROLLING_SPEED};

const FLAPPY_SPRITE_SIZE: f32 = 24.0;
const FLAPPY_SPRITE_SCALE: Vec3 = Vec3::splat(2.0);
const FLAPPY_SIZE: Vec3 = Vec3::new(
    FLAPPY_SPRITE_SCALE.x * FLAPPY_SPRITE_SIZE,
    FLAPPY_SPRITE_SCALE.y * FLAPPY_SPRITE_SIZE,
    0.0,
);
const FLAPPY_COLLISION_SIZE: Vec3 = Vec3::new(FLAPPY_SIZE.x * 0.65, FLAPPY_SIZE.y * 0.65, 0.0);
const FLAPPY_JUMP_STRENGTH: f32 = 700.0;
// Max height flappy can jump above the window height
const FLAPPY_MAX_FLY_HEIGHT: f32 = (WINDOW_HEIGHT / 2.0) + WINDOW_BOUND_LIMIT;

#[derive(Component)]
pub struct Flappy;

#[derive(Component)]
pub struct FlappyCollider {
    pub enabled: bool,
}

pub struct FlappyPlugin;

impl Plugin for FlappyPlugin {
    fn build(&self, app: &mut App) {
        use bevy::transform::TransformSystem;

        app.add_system_set(
            SystemSet::on_enter(GameState::Playing).with_system(flappy_setup_playing),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(flappy_jump)
                .with_system(flappy_limit_movement.after(ApplyVelocitySystem)),
        );
        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(check_for_collision)
                .after(TransformSystem::TransformPropagate),
        );
        app.add_system_set(
            SystemSet::on_enter(GameState::GameOver).with_system(flappy_forward_stop),
        );
    }
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_map: ResMut<Assets<TextureAtlas>>,
    position: Vec3,
) {
    let texture_handle = asset_server.load("characters.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::splat(FLAPPY_SPRITE_SIZE), 9, 3);
    let texture_atlas_handle = texture_atlas_map.add(texture_atlas);

    commands
        .spawn()
        .insert(Name::new("Flappy"))
        .insert(Flappy)
        .insert(Velocity(Vec2::ZERO))
        .insert(GravityAffected(false))
        .insert(FlappyCollider { enabled: true })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                translation: position,
                scale: FLAPPY_SPRITE_SCALE,
                ..default()
            },
            sprite: TextureAtlasSprite {
                flip_x: true,
                ..default()
            },
            ..default()
        })
        .insert(Animation {
            timer: Timer::from_seconds(0.3, true),
            frames: vec![25, 26, 24],
            current_frame: 0,
        });
}

//
// -- System
//

fn flappy_setup_playing(
    mut query: Query<(&mut Velocity, &mut GravityAffected, &mut Animation), With<Flappy>>,
) {
    let (mut velocity, mut gravity_affected, mut animation) = query.single_mut();

    velocity.0 = Vec2::new(SCROLLING_SPEED, 0.0);
    gravity_affected.0 = true;
    animation.timer.set_repeating(false);
}

fn flappy_jump(
    mut replay_event: EventWriter<AnimationReplayEvent>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity), With<Flappy>>,
) {
    let (flappy_entity, mut flappy_velocity) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        flappy_velocity.y = FLAPPY_JUMP_STRENGTH;
        replay_event.send(AnimationReplayEvent(flappy_entity));
    }
}

fn flappy_limit_movement(mut query: Query<&mut Transform, With<Flappy>>) {
    let mut flappy_transform = query.single_mut();

    if flappy_transform.translation.y > FLAPPY_MAX_FLY_HEIGHT {
        flappy_transform.translation.y = FLAPPY_MAX_FLY_HEIGHT;
    }
}

fn flappy_forward_stop(mut query: Query<(&mut Velocity, &mut FlappyCollider), With<Flappy>>) {
    let (mut flappy_velocity, mut flappy_collider) = query.single_mut();

    flappy_velocity.x = 0.0;
    flappy_collider.enabled = false;
}

fn check_for_collision(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut run_state: ResMut<State<GameState>>,
    flappy_query: Query<(&Transform, &FlappyCollider), With<Flappy>>,
    collider_query: Query<(Entity, &GlobalTransform, &Collider, Option<&PipeGap>)>,
) {
    let (flappy_transform, flappy_collider) = flappy_query.single();

    for (collider_entity, collider_transform, collider, maybe_pipe_gap) in &collider_query {
        let collider_translation = collider_transform.translation();
        let collider_relative_position = collider.position();
        let collider_position = Vec3::new(
            collider_translation.x + collider_relative_position.x,
            collider_translation.y + collider_relative_position.y,
            collider_translation.z + collider_relative_position.z,
        );

        let collision = collide_aabb::collide(
            flappy_transform.translation,
            FLAPPY_COLLISION_SIZE.truncate(),
            collider_position,
            *collider.scale(),
        );

        if collision.is_some() && flappy_collider.enabled {
            if maybe_pipe_gap.is_some() {
                scoreboard.update_current_score(1);
                commands.entity(collider_entity).despawn();
            } else {
                run_state.set(GameState::GameOver).unwrap();
            }
        }
    }
}
