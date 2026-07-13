use macroquad::prelude::*;

use crate::audio::Sounds;
use crate::game::{Game, GameMode, UpdateResult};
use crate::input::{
    InputAction, handle_countdown, handle_game_over, handle_paused, handle_playing,
    handle_start_menu,
};
use crate::persist::{Leaderboard, load_sound_enabled, save_sound_enabled};
use crate::render::{
    VisualState, draw_countdown, draw_game_over, draw_paused, draw_playing, draw_start_menu,
};

const COUNTDOWN_SECONDS: f32 = 3.0;

pub enum AppState {
    StartMenu,
    Countdown { remaining: f32, game: Game },
    Playing { game: Game, tick_elapsed: f32 },
    Paused { game: Game, tick_elapsed: f32 },
    GameOver { game: Game, is_new_record: bool },
}

pub struct App {
    pub state: AppState,
    pub selected_mode: GameMode,
    pub player_name: String,
    pub leaderboard: Leaderboard,
    pub visuals: VisualState,
    pub sound_enabled: bool,
    pub sounds: Sounds,
}

impl App {
    pub fn new(sounds: Sounds) -> Self {
        App {
            state: AppState::StartMenu,
            selected_mode: GameMode::Classic,
            player_name: String::new(),
            leaderboard: Leaderboard::load(),
            visuals: VisualState::default(),
            sound_enabled: load_sound_enabled(),
            sounds,
        }
    }

    fn toggle_sound(&mut self) {
        self.sound_enabled = !self.sound_enabled;
        save_sound_enabled(self.sound_enabled);
        if self.sound_enabled {
            self.sounds.play_eat(true);
        }
    }

    /// Returns `false` when the app should exit.
    pub fn update(&mut self) -> bool {
        let dt = get_frame_time();
        self.visuals.update(dt);

        match std::mem::replace(&mut self.state, AppState::StartMenu) {
            AppState::StartMenu => self.update_start_menu(),
            AppState::Countdown { remaining, game } => self.update_countdown(dt, remaining, game),
            AppState::Playing { game, tick_elapsed } => self.update_playing(dt, game, tick_elapsed),
            AppState::Paused { game, tick_elapsed } => self.update_paused(game, tick_elapsed),
            AppState::GameOver {
                game,
                is_new_record,
            } => self.update_game_over(game, is_new_record),
        }
    }

    pub fn draw(&self) {
        match &self.state {
            AppState::StartMenu => {
                draw_start_menu(
                    self.selected_mode,
                    &self.leaderboard,
                    &self.player_name,
                    self.sound_enabled,
                );
            }
            AppState::Countdown { remaining, .. } => draw_countdown(*remaining),
            AppState::Playing { game, tick_elapsed } => {
                let alpha = (*tick_elapsed / game.tick_seconds()).min(1.0);
                draw_playing(game, &self.visuals, alpha, self.leaderboard.top_score());
            }
            AppState::Paused { game, tick_elapsed } => {
                let alpha = (*tick_elapsed / game.tick_seconds()).min(1.0);
                draw_paused(game, &self.visuals, alpha, self.leaderboard.top_score(), self.sound_enabled);
            }
            AppState::GameOver {
                game,
                is_new_record,
            } => {
                draw_game_over(game, &self.leaderboard, *is_new_record);
            }
        }
    }

    fn update_start_menu(&mut self) -> bool {
        match handle_start_menu(&mut self.player_name) {
            InputAction::Quit => return false,
            InputAction::Start => {
                self.start_countdown();
                return true;
            }
            InputAction::ToggleMode => self.selected_mode = self.selected_mode.toggle(),
            InputAction::ToggleSound => self.toggle_sound(),
            _ => {}
        }
        self.state = AppState::StartMenu;
        true
    }

    fn update_countdown(&mut self, dt: f32, mut remaining: f32, game: Game) -> bool {
        if matches!(handle_countdown(), InputAction::Quit) {
            return false;
        }

        remaining -= dt;
        if remaining <= 0.0 {
            self.begin_play(game);
        } else {
            self.state = AppState::Countdown { remaining, game };
        }
        true
    }

    fn update_playing(&mut self, dt: f32, mut game: Game, mut tick_elapsed: f32) -> bool {
        match handle_playing(&mut game) {
            InputAction::Quit => return false,
            InputAction::Pause => {
                self.state = AppState::Paused { game, tick_elapsed };
                return true;
            }
            _ => {}
        }

        tick_elapsed += dt;
        let tick_secs = game.tick_seconds();
        while tick_elapsed >= tick_secs {
            tick_elapsed -= tick_secs;
            self.visuals.snapshot_body(&game.snake.body);

            match game.update() {
                UpdateResult::Died => {
                    self.visuals.trigger_death();
                    self.sounds.play_game_over(self.sound_enabled);
                    let is_new_record = self
                        .leaderboard
                        .add_score(game.score, &self.player_name);
                    self.state = AppState::GameOver {
                        game,
                        is_new_record,
                    };
                    return true;
                }
                UpdateResult::Ate => {
                    self.visuals.trigger_eat();
                    self.sounds.play_eat(self.sound_enabled);
                }
                _ => {}
            }
        }

        self.state = AppState::Playing { game, tick_elapsed };
        true
    }

    fn update_paused(&mut self, game: Game, tick_elapsed: f32) -> bool {
        match handle_paused() {
            InputAction::Quit => return false,
            InputAction::Resume => {
                self.state = AppState::Playing { game, tick_elapsed };
            }
            InputAction::ToggleSound => {
                self.toggle_sound();
                self.state = AppState::Paused { game, tick_elapsed };
            }
            _ => {
                self.state = AppState::Paused { game, tick_elapsed };
            }
        }
        true
    }

    fn update_game_over(&mut self, game: Game, is_new_record: bool) -> bool {
        match handle_game_over() {
            InputAction::Quit => return false,
            InputAction::PlayAgain => {
                self.start_countdown();
                return true;
            }
            _ => {}
        }
        self.state = AppState::GameOver {
            game,
            is_new_record,
        };
        true
    }

    fn start_countdown(&mut self) {
        let game = Game::new(self.selected_mode);
        self.visuals = VisualState::default();
        self.visuals.snapshot_body(&game.snake.body);
        self.state = AppState::Countdown {
            remaining: COUNTDOWN_SECONDS,
            game,
        };
    }

    fn begin_play(&mut self, game: Game) {
        self.visuals = VisualState::default();
        self.visuals.snapshot_body(&game.snake.body);
        self.state = AppState::Playing {
            game,
            tick_elapsed: 0.0,
        };
    }
}
