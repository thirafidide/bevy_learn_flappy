use bevy::prelude::*;
use rand::Rng;

use crate::collider::Collider;
use crate::game_state::GameState;
use crate::window::*;

const PIPE_GAP: f32 = 200.0;
pub const PIPE_WIDTH: f32 = 125.0;
const PIPE_GAP_MIN_Y: f32 = -200.0;
const PIPE_GAP_MAX_Y: f32 = 200.0;
const PIPE_COLOR: Color = Color::rgb(0.6, 0.85, 0.4);

const PIPE_SET_ENTITY_COUNT: u32 = 3;
const PIPE_DISTANCE: f32 = 350.0;
const DISTANCE_TO_FIRST_PIPE: f32 = 500.0;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Playing).with_system(side_scroll));
    }
}

//
// -- COMPONENT
//

#[derive(Debug)]
enum PipePosition {
    Top,
    Bottom,
}

#[derive(Debug, Component)]
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
                    Vec3::new(0.0, pipe_y, 0.0),
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
                    Vec3::new(0.0, pipe_y, 0.0),
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
}

#[derive(Component)]
pub struct PipeSet;

#[derive(Bundle)]
pub struct PipeSetBundle {
    pipe_set: PipeSet,
    #[bundle]
    transform: TransformBundle,
    #[bundle]
    visibility: VisibilityBundle,
}

impl PipeSetBundle {
    fn new(position_x: f32) -> PipeSetBundle {
        PipeSetBundle {
            pipe_set: PipeSet,
            transform: TransformBundle::from_transform(Transform::from_translation(Vec3::new(
                position_x, 0.0, 1.0,
            ))),
            visibility: VisibilityBundle::default(),
        }
    }

    pub fn spawn(commands: &mut Commands, position_x: f32) {
        let gap_position = Vec2::new(
            position_x,
            rand::thread_rng().gen_range(PIPE_GAP_MIN_Y..=PIPE_GAP_MAX_Y),
        );

        let (top_pipe, bottom_pipe) = PipeBundle::new_set(&gap_position);

        commands
            .spawn()
            .insert(PipeSet)
            .insert(Name::new("Pipe Set"))
            .insert_bundle(Self::new(position_x))
            .with_children(|parent| {
                parent
                    .spawn()
                    .insert(Name::new("Top Pipe"))
                    .insert_bundle(top_pipe);

                parent
                    .spawn()
                    .insert(Name::new("Bottom Pipe"))
                    .insert_bundle(bottom_pipe);
            });
    }
}

//
// -- SYSTEM
//

pub fn setup(commands: &mut Commands) {
    for i in 0..PIPE_SET_ENTITY_COUNT {
        let gap_position_x = DISTANCE_TO_FIRST_PIPE + (PIPE_DISTANCE * (i as f32));
        PipeSetBundle::spawn(commands, gap_position_x)
    }
}

pub fn side_scroll(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    pipe_sets_query: Query<(Entity, &Transform), (With<PipeSet>, Without<Camera2d>)>,
) {
    let camera_transform = camera_query.single();

    // when a pipe moved out of sight, despawn it and spawn a new one at the back
    for (pipe_sets_entity, pipe_transform) in &pipe_sets_query {
        let pipe_right_edge_position = pipe_transform.translation.x + (PIPE_WIDTH / 2.0);
        let camera_left_edge_position = camera_transform.translation.x - (WINDOW_WIDTH / 2.0);

        if pipe_right_edge_position + WINDOW_BOUND_LIMIT < camera_left_edge_position {
            let new_gap_position_x =
                pipe_transform.translation.x + PIPE_DISTANCE * (PIPE_SET_ENTITY_COUNT as f32);

            PipeSetBundle::spawn(&mut commands, new_gap_position_x);
            commands.entity(pipe_sets_entity).despawn_recursive();
        }
    }
}
