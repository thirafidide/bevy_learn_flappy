use crate::velocity::Velocity;
use bevy::prelude::*;

const GRAVITY: f32 = 2400.0;

#[derive(Component)]
pub struct GravityAffected(pub bool);

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_gravity);
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<(&mut Velocity, &GravityAffected)>) {
    for (mut velocity, gravity_affected) in query.iter_mut() {
        if gravity_affected.0 {
            velocity.y -= GRAVITY * time.delta_seconds();
        }
    }
}
