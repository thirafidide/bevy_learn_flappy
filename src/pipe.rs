use bevy::prelude::*;
use rand::Rng;

use crate::collider::Collider;
use crate::window::*;

const PIPE_GAP: f32 = 200.0;
pub const PIPE_WIDTH: f32 = 125.0;
const PIPE_GAP_MIN_Y: f32 = -200.0;
const PIPE_GAP_MAX_Y: f32 = 200.0;
const PIPE_COLOR: Color = Color::rgb(0.6, 0.85, 0.4);

enum PipePosition {
    Top,
    Bottom,
}

#[derive(Component)]
pub struct Pipe {
    position: PipePosition,
}

impl Pipe {
    fn construct_sprite_bundle(translation: Vec3, scale: Vec3) -> SpriteBundle {
        SpriteBundle {
            transform: Transform {
                translation,
                scale,
                ..default()
            },
            sprite: Sprite {
                color: PIPE_COLOR,
                ..default()
            },
            ..default()
        }
    }

    fn sprite_bundle(&self, gap_center: &Vec2) -> SpriteBundle {
        match self.position {
            PipePosition::Top => {
                let pipe_bottom_y = gap_center.y + PIPE_GAP / 2.0;
                let window_top = WINDOW_HEIGHT / 2.0;
                let height_to_top = window_top - pipe_bottom_y;
                let pipe_height = height_to_top + WINDOW_BOUND_LIMIT;
                let pipe_y = pipe_bottom_y + pipe_height / 2.0;

                Self::construct_sprite_bundle(
                    Vec3::new(gap_center.x, pipe_y, 0.0),
                    Vec3::new(PIPE_WIDTH, pipe_height, 0.0),
                )
            }

            PipePosition::Bottom => {
                let pipe_top_y = gap_center.y - PIPE_GAP / 2.0;
                let window_bottom = -WINDOW_HEIGHT / 2.0;
                let height_to_bottom = pipe_top_y - window_bottom;
                let pipe_height = height_to_bottom + WINDOW_BOUND_LIMIT;
                let pipe_y = pipe_top_y - pipe_height / 2.0;

                Self::construct_sprite_bundle(
                    Vec3::new(gap_center.x, pipe_y, 0.0),
                    Vec3::new(PIPE_WIDTH, pipe_height, 0.0),
                )
            }
        }
    }
}

#[derive(Bundle)]
pub struct PipeBundle {
    #[bundle]
    sprite: SpriteBundle,
    collider: Collider,
    pipe: Pipe,
}

impl PipeBundle {
    pub fn new_set(gap_center: &Vec2) -> (Self, Self) {
        let top_pipe = Pipe {
            position: PipePosition::Top,
        };
        let bottom_pipe = Pipe {
            position: PipePosition::Bottom,
        };

        (
            PipeBundle {
                sprite: top_pipe.sprite_bundle(gap_center),
                collider: Collider::new(
                    top_pipe
                        .sprite_bundle(gap_center)
                        .transform
                        .scale
                        .truncate(),
                ),
                pipe: top_pipe,
            },
            PipeBundle {
                sprite: bottom_pipe.sprite_bundle(gap_center),
                collider: Collider::new(
                    bottom_pipe
                        .sprite_bundle(gap_center)
                        .transform
                        .scale
                        .truncate(),
                ),
                pipe: bottom_pipe,
            },
        )
    }

    pub fn spawn_set(commands: &mut Commands, position_x: f32) {
        let gap_position = Vec2::new(
            position_x,
            rand::thread_rng().gen_range(PIPE_GAP_MIN_Y..=PIPE_GAP_MAX_Y),
        );

        let (top_pipe, bottom_pipe) = Self::new_set(&gap_position);
        commands.spawn_bundle(top_pipe);
        commands.spawn_bundle(bottom_pipe);
    }
}
