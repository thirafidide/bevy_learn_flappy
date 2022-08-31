#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Playing,
    GameOver,
    Cleanup,
}
