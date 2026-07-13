use crate::game::{BoardPreset, DeathCause, Game, GameConfig, GameMode, UpdateResult};
use crate::persist::{Achievements, Leaderboards};
use crate::render::VisualState;

pub struct RunSession {
    pub game: Game,
    pub visuals: VisualState,
    pub tick_elapsed: f32,
    pub previous_best: u16,
}

impl RunSession {
    pub fn new(mode: GameMode, preset: BoardPreset) -> Self {
        let config = match mode {
            GameMode::Daily => GameConfig::daily(),
            _ => preset.config(),
        };
        let game = if mode == GameMode::Daily {
            Game::new_with_seed(mode, config, Some(Game::daily_seed_for_today()))
        } else {
            Game::new(mode, config)
        };
        let mut visuals = VisualState::default();
        visuals.snapshot_body(&game.snake.body);
        RunSession {
            game,
            visuals,
            tick_elapsed: 0.0,
            previous_best: 0,
        }
    }

    pub fn set_previous_best(&mut self, best: u16) {
        self.previous_best = best;
    }
}

pub struct PlayerProfile {
    pub name: String,
    pub name_editing: bool,
    pub sound_enabled: bool,
    pub selected_mode: GameMode,
    pub selected_preset: BoardPreset,
    pub leaderboards: Leaderboards,
    pub achievements: Achievements,
}

impl PlayerProfile {
    pub fn new() -> Self {
        PlayerProfile {
            name: String::new(),
            name_editing: true,
            sound_enabled: crate::persist::load_sound_enabled(),
            selected_mode: GameMode::Classic,
            selected_preset: BoardPreset::Classic,
            leaderboards: Leaderboards::load(),
            achievements: Achievements::load(),
        }
    }
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure tick advancement for testable FSM logic.
pub struct TickOutcome {
    pub result: Option<GameTickEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTickEvent {
    Ate,
    LeveledUp,
    Died(DeathCause),
    Won,
}

pub fn advance_session_ticks(session: &mut RunSession, dt: f32) -> TickOutcome {
    session.tick_elapsed += dt;
    let tick_secs = session.game.tick_seconds();
    let mut event = None;

    while session.tick_elapsed >= tick_secs {
        session.tick_elapsed -= tick_secs;
        session.visuals.snapshot_body(&session.game.snake.body);

        match session.game.update() {
            UpdateResult::Died(cause) => {
                event = Some(GameTickEvent::Died(cause));
                break;
            }
            UpdateResult::Won => {
                event = Some(GameTickEvent::Won);
                break;
            }
            UpdateResult::Ate => {
                if session.game.last_ate.as_ref().is_some_and(|a| a.leveled_up) {
                    event = Some(GameTickEvent::LeveledUp);
                } else {
                    event = Some(GameTickEvent::Ate);
                }
                session.visuals.trigger_eat();
            }
            UpdateResult::Moved => {}
        }
    }

    TickOutcome { result: event }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{DeathCause, FoodKind};

    #[test]
    fn advance_ticks_detects_death() {
        let mut session = RunSession::new(GameMode::Classic, BoardPreset::Classic);
        session.game.snake.body = vec![(0, 0)];
        session.game.snake.direction = crate::game::Direction::Left;
        let outcome = advance_session_ticks(&mut session, 1.0);
        assert_eq!(outcome.result, Some(GameTickEvent::Died(DeathCause::Wall)));
    }

    #[test]
    fn advance_ticks_detects_eat() {
        let mut session = RunSession::new(GameMode::Classic, BoardPreset::Classic);
        let next = crate::game::Snake::resolve_head(
            session.game.snake.next_head_coords(),
            GameMode::Classic,
            session.game.config.board_width,
            session.game.config.board_height,
        )
        .unwrap();
        session.game.food.position = next;
        session.game.food.kind = FoodKind::Normal;
        let outcome = advance_session_ticks(&mut session, 1.0);
        assert_eq!(outcome.result, Some(GameTickEvent::Ate));
    }

    #[test]
    fn advance_ticks_detects_win_on_full_board() {
        let mut session = RunSession::new(GameMode::Classic, BoardPreset::Classic);
        session.game.config.board_width = 3;
        session.game.config.board_height = 2;
        session.game.snake.body = vec![(1, 1), (0, 1), (0, 0), (1, 0), (2, 0)];
        session.game.snake.direction = crate::game::Direction::Right;
        session.game.food.position = (2, 1);
        session.game.food.kind = FoodKind::Normal;
        let outcome = advance_session_ticks(&mut session, 1.0);
        assert_eq!(outcome.result, Some(GameTickEvent::Won));
    }

    #[test]
    fn death_cause_wall_vs_self() {
        let mut session = RunSession::new(GameMode::Classic, BoardPreset::Classic);
        session.game.snake.body = vec![(0, 0)];
        session.game.snake.direction = crate::game::Direction::Left;
        assert!(matches!(
            session.game.update(),
            UpdateResult::Died(DeathCause::Wall)
        ));
    }
}
