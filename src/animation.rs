use bevy::prelude::*;

#[derive(Component)]
pub struct Animation {
    pub current_frame: usize,
    pub timer: Timer,
    pub frames: Vec<usize>,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationReplayEvent>();
        app.add_system(play_animation);
        app.add_system(replay_animation);
    }
}

pub struct AnimationReplayEvent(pub Entity);

fn play_animation(time: Res<Time>, mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.timer.tick(time.delta());
        animation.current_frame =
            ((animation.timer.percent() - 0.01) * (animation.frames.len() as f32)).floor() as usize;
        sprite.index = animation.frames[animation.current_frame];
    }
}

fn replay_animation(
    mut reset_events: EventReader<AnimationReplayEvent>,
    mut query: Query<(Entity, &mut Animation)>,
) {
    for reset_event in reset_events.iter() {
        for (entity, mut animation) in query.iter_mut() {
            if entity == reset_event.0 {
                animation.current_frame = 0;
                animation.timer.reset();
                animation.timer.unpause();
            }
        }
    }
}
