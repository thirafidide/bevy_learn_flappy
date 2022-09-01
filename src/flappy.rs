use bevy::prelude::*;
use bevy::sprite::collide_aabb::{self, Collision};

use crate::animation::Animation;
use crate::gravity::GravityAffected;
use crate::velocity::Velocity;

const FLAPPY_SPRITE_SIZE: f32 = 24.0;
const FLAPPY_SPRITE_SCALE: Vec3 = Vec3::splat(2.0);
const FLAPPY_SIZE: Vec3 = Vec3::new(
    FLAPPY_SPRITE_SCALE.x * FLAPPY_SPRITE_SIZE,
    FLAPPY_SPRITE_SCALE.y * FLAPPY_SPRITE_SIZE,
    0.0,
);
const FLAPPY_COLLISION_SIZE: Vec3 = Vec3::new(FLAPPY_SIZE.x * 0.65, FLAPPY_SIZE.y * 0.65, 0.0);
const FLAPPY_JUMP_STRENGTH: f32 = 700.0;

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
        .insert(GravityAffected)
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
            timer: Timer::from_seconds(0.3, false),
            frames: vec![25, 26, 24],
            current_frame: 0,
        });
}

//
// -- System
//

pub fn jump(mut velocity: Mut<Velocity>) {
    velocity.y = FLAPPY_JUMP_STRENGTH;
}

pub fn collide(transform: &Transform, other_pos: Vec3, other_size: Vec2) -> Option<Collision> {
    collide_aabb::collide(
        transform.translation,
        FLAPPY_COLLISION_SIZE.truncate(),
        other_pos,
        other_size,
    )
}
