use bevy::prelude::*;

use crate::collider::Collider;
use crate::window::*;

pub const FLOOR_WIDTH: f32 = WINDOW_WIDTH;
const FLOOR_THICKNESS: f32 = 30.0;
const FLOOR_POSITION_Y: f32 = -WINDOW_HEIGHT / 2.0 + (FLOOR_THICKNESS / 2.0);
const FLOOR_STARTING_POSITION_X: f32 = -WINDOW_WIDTH / 2.0;
const FLOOR_COLOR: Color = Color::rgb(0.5, 0.5, 0.7);

#[derive(Component)]
pub struct Floor;

#[derive(Bundle)]
pub struct FloorBundle {
    #[bundle]
    sprite: SpriteBundle,
    collider: Collider,
    floor: Floor,
}

impl FloorBundle {
    pub fn new(index: u32) -> Self {
        let pos = index as f32;
        let translation_x = FLOOR_STARTING_POSITION_X + (FLOOR_WIDTH / 2.0) + (pos * FLOOR_WIDTH);

        FloorBundle {
            sprite: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(translation_x, FLOOR_POSITION_Y, 2.0),
                    scale: Vec3::new(FLOOR_WIDTH, FLOOR_THICKNESS, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: FLOOR_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            floor: Floor,
        }
    }
}
