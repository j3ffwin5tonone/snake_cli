use macroquad::prelude::*;

use crate::game::{Direction, Game};

pub enum InputAction {
    None,
    Quit,
    Start,
    ToggleMode,
    Pause,
    Resume,
    PlayAgain,
}

pub fn handle_start_menu() -> InputAction {
    if quit_pressed() {
        return InputAction::Quit;
    }
    if is_key_pressed(KeyCode::Enter) {
        return InputAction::Start;
    }
    if is_key_pressed(KeyCode::M) {
        return InputAction::ToggleMode;
    }
    InputAction::None
}

pub fn handle_countdown() -> InputAction {
    if quit_pressed() {
        InputAction::Quit
    } else {
        InputAction::None
    }
}

pub fn handle_playing(game: &mut Game) -> InputAction {
    if quit_pressed() {
        return InputAction::Quit;
    }
    if is_key_pressed(KeyCode::P) {
        return InputAction::Pause;
    }
    if let Some(dir) = steering_pressed() {
        game.snake.change_direction(dir);
    }
    InputAction::None
}

pub fn handle_paused() -> InputAction {
    if quit_pressed() {
        return InputAction::Quit;
    }
    if is_key_pressed(KeyCode::P) {
        return InputAction::Resume;
    }
    InputAction::None
}

pub fn handle_game_over() -> InputAction {
    if quit_pressed() {
        return InputAction::Quit;
    }
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::R) {
        return InputAction::PlayAgain;
    }
    InputAction::None
}

fn quit_pressed() -> bool {
    is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape)
}

fn steering_pressed() -> Option<Direction> {
    if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
        Some(Direction::Up)
    } else if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
        Some(Direction::Down)
    } else if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
        Some(Direction::Left)
    } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
        Some(Direction::Right)
    } else {
        None
    }
}
