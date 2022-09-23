use bevy::prelude::*;

use crate::{flappy::Flappy, game_state::GameState};

// TODO duplicate from main.rs, remove once score update
// refactored to do proper update on gap collusion
const PIPE_DISTANCE: f32 = 350.0;
const DISTANCE_TO_FIRST_PIPE: f32 = 500.0;

#[derive(Debug, Clone)]
pub struct Scoreboard {
    current_score: u32,
    best_score: u32,
}

impl Scoreboard {
    fn new() -> Self {
        Self {
            current_score: 0,
            best_score: 0,
        }
    }

    fn update_current_score(&mut self, new_score: u32) {
        self.current_score = new_score;
    }

    fn update_best_score(&mut self) {
        if self.current_score > self.best_score {
            self.best_score = self.current_score
        }
    }

    fn reset_current_score(&mut self) {
        self.current_score = 0;
    }
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scoreboard::new()).add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(update_current_score),
        );
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(update_best_score));
        app.add_system_set(
            SystemSet::on_exit(GameState::GameOver).with_system(reset_current_score),
        );
    }
}

fn reset_current_score(mut scoreboard: ResMut<Scoreboard>) {
    scoreboard.reset_current_score();
}

fn update_current_score(
    mut scoreboard: ResMut<Scoreboard>,
    flappy_query: Query<&Transform, With<Flappy>>,
) {
    let flappy_transform = flappy_query.single();

    scoreboard.update_current_score(
        ((flappy_transform.translation.x - DISTANCE_TO_FIRST_PIPE) / PIPE_DISTANCE).round() as u32,
    );
}

fn update_best_score(mut scoreboard: ResMut<Scoreboard>) {
    scoreboard.update_best_score();
}
