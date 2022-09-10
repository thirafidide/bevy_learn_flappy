use bevy::prelude::*;

use crate::collider::Collider;
use crate::game_state::GameState;
use crate::window::*;

const FLOOR_WIDTH: f32 = WINDOW_WIDTH;
const FLOOR_THICKNESS: f32 = 30.0;
const FLOOR_POSITION_Y: f32 = -WINDOW_HEIGHT / 2.0 + (FLOOR_THICKNESS / 2.0);
const FLOOR_STARTING_POSITION_X: f32 = -WINDOW_WIDTH / 2.0;
const FLOOR_COLOR: Color = Color::rgb(0.5, 0.5, 0.7);
// for infinite floor, 3 floor entities reused when one move out of the window
const FLOOR_ENTITY_COUNT: u32 = 3;

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Playing).with_system(side_scroll));
    }
}

//
// -- Component
//

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
        let scale = Vec3::new(FLOOR_WIDTH, FLOOR_THICKNESS, 0.0);

        FloorBundle {
            sprite: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(translation_x, FLOOR_POSITION_Y, 2.0),
                    scale,
                    ..default()
                },
                sprite: Sprite {
                    color: FLOOR_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider::new(scale.truncate()),
            floor: Floor,
        }
    }
}

//
// -- SYSTEM
//

pub fn setup(commands: &mut Commands) {
    for i in 0..FLOOR_ENTITY_COUNT {
        commands
            .spawn()
            .insert(Name::new("Floor"))
            .insert_bundle(FloorBundle::new(i));
    }
}

fn side_scroll(
    camera_query: Query<&Transform, With<Camera2d>>,
    mut floor_query: Query<&mut Transform, (With<Floor>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

    // when a floor moved out of sight, reuse it by moving it to the back
    for mut floor_transform in &mut floor_query {
        let floor_right_edge_position = floor_transform.translation.x + (FLOOR_WIDTH / 2.0);
        let camera_left_edge_position = camera_transform.translation.x - (WINDOW_WIDTH / 2.0);

        if floor_right_edge_position + WINDOW_BOUND_LIMIT < camera_left_edge_position {
            floor_transform.translation.x += FLOOR_WIDTH * (FLOOR_ENTITY_COUNT as f32);
        }
    }
}
