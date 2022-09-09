use bevy::prelude::*;

#[derive(Component)]
pub struct Collider {
    /// Collider box position relative to the entity's transform
    position: Vec3,
    /// Collider box scale
    scale: Vec2,
}

impl Collider {
    pub fn new(scale: Vec2) -> Self {
        Collider {
            position: Vec3::ZERO,
            scale,
        }
    }

    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    pub fn scale(&self) -> &Vec2 {
        &self.scale
    }
}
