use macroquad::prelude::*;

use crate::audio::Sounds;
use crate::game::DeathCause;
use crate::input::{
    InputAction, handle_countdown, handle_game_over, handle_paused, handle_playing,
    handle_start_menu,
};
use crate::persist::save_sound_enabled;
use crate::render::{
    draw_countdown, draw_end_screen, draw_level_up, draw_paused, draw_playing, draw_start_menu,
};
use crate::session::{GameTickEvent, PlayerProfile, RunSession, advance_session_ticks};

const COUNTDOWN_SECONDS: f32 = 3.0;
const LEVEL_UP_DISPLAY_SECS: f32 = 0.8;

pub enum AppState {
    StartMenu,
    Countdown {
        remaining: f32,
        session: RunSession,
    },
    Playing {
        session: RunSession,
        level_up_timer: f32,
    },
    Paused {
        session: RunSession,
        level_up_timer: f32,
    },
    EndScreen {
        session: RunSession,
        is_new_record: bool,
        won: bool,
        death_cause: Option<DeathCause>,
        new_achievements: Vec<&'static crate::persist::AchievementDef>,
    },
}

pub struct App {
    pub state: AppState,
    pub profile: PlayerProfile,
    pub sounds: Sounds,
}

impl App {
    pub fn new(sounds: Sounds) -> Self {
        App {
            state: AppState::StartMenu,
            profile: PlayerProfile::new(),
            sounds,
        }
    }

    fn toggle_sound(&mut self) {
        self.profile.sound_enabled = !self.profile.sound_enabled;
        save_sound_enabled(self.profile.sound_enabled);
        if self.profile.sound_enabled {
            self.sounds.play_eat(true);
        }
    }

    pub fn update(&mut self) -> bool {
        let dt = get_frame_time();
        self.update_visuals(dt);

        match std::mem::replace(&mut self.state, AppState::StartMenu) {
            AppState::StartMenu => self.update_start_menu(),
            AppState::Countdown { remaining, session } => {
                self.update_countdown(dt, remaining, session)
            }
            AppState::Playing {
                session,
                level_up_timer,
            } => self.update_playing(dt, session, level_up_timer),
            AppState::Paused {
                session,
                level_up_timer,
            } => self.update_paused(session, level_up_timer),
            AppState::EndScreen {
                session,
                is_new_record,
                won,
                death_cause,
                new_achievements,
            } => self.update_end_screen(session, is_new_record, won, death_cause, new_achievements),
        }
    }

    fn update_visuals(&mut self, dt: f32) {
        match &mut self.state {
            AppState::Playing { session, .. } | AppState::Paused { session, .. } => {
                session.visuals.update(dt);
            }
            AppState::Countdown { session, .. } => session.visuals.update(dt),
            _ => {}
        }
    }

    pub fn draw(&self) {
        match &self.state {
            AppState::StartMenu => draw_start_menu(
                self.profile.selected_mode,
                self.profile.selected_preset,
                &self.profile.leaderboards,
                &self.profile.name,
                self.profile.name_editing,
                self.profile.sound_enabled,
                &self.profile.achievements,
            ),
            AppState::Countdown { remaining, .. } => draw_countdown(*remaining),
            AppState::Playing {
                session,
                level_up_timer,
            } => {
                let alpha = (session.tick_elapsed / session.game.tick_seconds()).min(1.0);
                let top = self.profile.leaderboards.top_score_for(session.game.mode);
                draw_playing(&session.game, &session.visuals, alpha, top);
                if *level_up_timer > 0.0 {
                    draw_level_up(*level_up_timer);
                }
            }
            AppState::Paused {
                session,
                level_up_timer,
            } => {
                let alpha = (session.tick_elapsed / session.game.tick_seconds()).min(1.0);
                let top = self.profile.leaderboards.top_score_for(session.game.mode);
                draw_paused(
                    &session.game,
                    &session.visuals,
                    alpha,
                    top,
                    self.profile.sound_enabled,
                );
                if *level_up_timer > 0.0 {
                    draw_level_up(*level_up_timer);
                }
            }
            AppState::EndScreen {
                session,
                is_new_record,
                won,
                death_cause,
                new_achievements,
            } => draw_end_screen(
                &session.game,
                &self.profile.leaderboards,
                *is_new_record,
                *won,
                *death_cause,
                session.previous_best,
                new_achievements,
            ),
        }
    }

    fn update_start_menu(&mut self) -> bool {
        match handle_start_menu(&mut self.profile.name, &mut self.profile.name_editing) {
            InputAction::Quit => return false,
            InputAction::Start => {
                self.start_countdown();
                return true;
            }
            InputAction::ToggleMode => {
                self.profile.selected_mode = self.profile.selected_mode.toggle();
            }
            InputAction::TogglePreset => {
                self.profile.selected_preset = self.profile.selected_preset.toggle();
            }
            InputAction::ToggleSound => self.toggle_sound(),
            _ => {}
        }
        self.state = AppState::StartMenu;
        true
    }

    fn update_countdown(&mut self, dt: f32, mut remaining: f32, session: RunSession) -> bool {
        if matches!(handle_countdown(), InputAction::Quit) {
            return false;
        }
        remaining -= dt;
        if remaining <= 0.0 {
            self.begin_play(session);
        } else {
            self.state = AppState::Countdown { remaining, session };
        }
        true
    }

    fn update_playing(
        &mut self,
        dt: f32,
        mut session: RunSession,
        mut level_up_timer: f32,
    ) -> bool {
        match handle_playing(&mut session.game) {
            InputAction::Quit => return false,
            InputAction::Pause => {
                self.state = AppState::Paused {
                    session,
                    level_up_timer,
                };
                return true;
            }
            _ => {}
        }

        if level_up_timer > 0.0 {
            level_up_timer = (level_up_timer - dt).max(0.0);
        }

        let outcome = advance_session_ticks(&mut session, dt);
        if let Some(event) = outcome.result {
            return match event {
                GameTickEvent::Died(cause) => self.finish_run(session, false, Some(cause)),
                GameTickEvent::Won => self.finish_run(session, true, None),
                GameTickEvent::Ate => {
                    self.sounds.play_eat(self.profile.sound_enabled);
                    self.state = AppState::Playing {
                        session,
                        level_up_timer,
                    };
                    true
                }
                GameTickEvent::LeveledUp => {
                    self.sounds.play_eat(self.profile.sound_enabled);
                    self.sounds.play_level_up(self.profile.sound_enabled);
                    session.visuals.trigger_level_up();
                    self.state = AppState::Playing {
                        session,
                        level_up_timer: LEVEL_UP_DISPLAY_SECS,
                    };
                    true
                }
            };
        }

        self.state = AppState::Playing {
            session,
            level_up_timer,
        };
        true
    }

    fn update_paused(&mut self, session: RunSession, level_up_timer: f32) -> bool {
        match handle_paused() {
            InputAction::Quit => return false,
            InputAction::Resume => {
                self.state = AppState::Playing {
                    session,
                    level_up_timer,
                };
            }
            InputAction::ToggleSound => {
                self.toggle_sound();
                self.state = AppState::Paused {
                    session,
                    level_up_timer,
                };
            }
            _ => {
                self.state = AppState::Paused {
                    session,
                    level_up_timer,
                };
            }
        }
        true
    }

    fn update_end_screen(
        &mut self,
        session: RunSession,
        is_new_record: bool,
        won: bool,
        death_cause: Option<DeathCause>,
        new_achievements: Vec<&'static crate::persist::AchievementDef>,
    ) -> bool {
        match handle_game_over() {
            InputAction::Quit => return false,
            InputAction::PlayAgain => {
                self.start_countdown();
                return true;
            }
            _ => {}
        }
        self.state = AppState::EndScreen {
            session,
            is_new_record,
            won,
            death_cause,
            new_achievements,
        };
        true
    }

    fn finish_run(
        &mut self,
        mut session: RunSession,
        won: bool,
        death_cause: Option<DeathCause>,
    ) -> bool {
        session.visuals.trigger_death();
        if won {
            self.sounds.play_level_up(self.profile.sound_enabled);
        } else {
            self.sounds.play_game_over(self.profile.sound_enabled);
        }

        let mode = session.game.mode;
        let score = session.game.score;
        let is_new_record = self
            .profile
            .leaderboards
            .add_score(mode, score, &self.profile.name);

        self.profile
            .achievements
            .evaluate_run(&session.game, won, death_cause);
        let new_achievements = self.profile.achievements.take_newly_unlocked();
        self.profile.achievements.persist();

        self.state = AppState::EndScreen {
            session,
            is_new_record,
            won,
            death_cause,
            new_achievements,
        };
        true
    }

    fn start_countdown(&mut self) {
        let mut session = RunSession::new(self.profile.selected_mode, self.profile.selected_preset);
        if session.game.mode == crate::game::GameMode::Daily {
            self.profile.leaderboards.daily_date_key = crate::game::Game::daily_date_key();
        }
        let best = self.profile.leaderboards.top_score_for(session.game.mode);
        session.set_previous_best(best);
        self.state = AppState::Countdown {
            remaining: COUNTDOWN_SECONDS,
            session,
        };
    }

    fn begin_play(&mut self, session: RunSession) {
        self.state = AppState::Playing {
            session,
            level_up_timer: 0.0,
        };
    }
}
