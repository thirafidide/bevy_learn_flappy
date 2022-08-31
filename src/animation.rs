use bevy::prelude::*;

#[derive(Component)]
pub struct Animation {
    pub timer: Timer,
    pub frames: Vec<usize>,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(play_animation);
    }
}

fn play_animation(time: Res<Time>, mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            let current_frame = animation.frames.iter().position(|&i| i == sprite.index);

            if let Some(current_frame) = current_frame {
                let next_frame = (current_frame + 1) % animation.frames.len();
                sprite.index = animation.frames[next_frame];
            }
        }
    }
}
