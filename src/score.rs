use bevy::prelude::*;

use crate::game_state::GameState;

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

    pub fn update_current_score(&mut self, increment: u32) {
        self.current_score += increment;
        println!("{}", self.current_score);
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
        app.insert_resource(Scoreboard::new());
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(update_best_score));
        app.add_system_set(
            SystemSet::on_exit(GameState::GameOver).with_system(reset_current_score),
        );
    }
}

fn reset_current_score(mut scoreboard: ResMut<Scoreboard>) {
    scoreboard.reset_current_score();
}

fn update_best_score(mut scoreboard: ResMut<Scoreboard>) {
    scoreboard.update_best_score();
}
