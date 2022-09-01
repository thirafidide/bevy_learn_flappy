use crate::velocity::Velocity;
use bevy::prelude::*;

const GRAVITY: f32 = 40.0;

#[derive(Component)]
pub struct GravityAffected;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_gravity);
    }
}

fn apply_gravity(mut query: Query<(&mut Velocity, &GravityAffected)>) {
    for (mut velocity, _) in query.iter_mut() {
        velocity.y -= GRAVITY;
    }
}
