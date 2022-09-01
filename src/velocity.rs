use bevy::prelude::*;

pub struct VelocityPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub struct ApplyVelocitySystem;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_velocity.label(ApplyVelocitySystem));
    }
}

#[derive(Component, Deref, DerefMut, Debug)]
pub struct Velocity(pub Vec2);

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x = transform.translation.x + velocity.x * time.delta_seconds();
        transform.translation.y = transform.translation.y + velocity.y * time.delta_seconds();
    }
}
