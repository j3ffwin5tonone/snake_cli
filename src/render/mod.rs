use macroquad::prelude::*;

use crate::game::GameConfig;

pub mod board;
pub mod effects;
pub mod hud;
pub mod screens;

pub use board::draw_playing;
pub use effects::{VisualState, draw_level_up};
pub use screens::{draw_countdown, draw_end_screen, draw_paused, draw_start_menu};

pub const CELL_SIZE: f32 = 30.0;
pub const UI_STRIP: f32 = 50.0;
pub const MARGIN: f32 = 20.0;
pub const MENU_EXTRA: f32 = 40.0;

pub const VIEWPORT_BOARD_W: u16 = 25;
pub const VIEWPORT_BOARD_H: u16 = 15;

pub const WINDOW_WIDTH: i32 = (VIEWPORT_BOARD_W as f32 * CELL_SIZE + MARGIN * 2.0) as i32;
pub const WINDOW_HEIGHT: i32 =
    (VIEWPORT_BOARD_H as f32 * CELL_SIZE + UI_STRIP + MARGIN * 2.0 + MENU_EXTRA) as i32;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

pub fn cell_size_for(config: &GameConfig) -> f32 {
    let max_w = VIEWPORT_BOARD_W as f32 * CELL_SIZE;
    let max_h = VIEWPORT_BOARD_H as f32 * CELL_SIZE;
    let board_w = config.board_width as f32 * CELL_SIZE;
    let board_h = config.board_height as f32 * CELL_SIZE;
    if board_w <= max_w && board_h <= max_h {
        CELL_SIZE
    } else {
        let scale_w = max_w / board_w;
        let scale_h = max_h / board_h;
        CELL_SIZE * scale_w.min(scale_h)
    }
}

pub fn board_offset(config: &GameConfig, cell_size: f32) -> (f32, f32) {
    let board_w = config.board_width as f32 * cell_size;
    let board_h = config.board_height as f32 * cell_size;
    let viewport_w = VIEWPORT_BOARD_W as f32 * CELL_SIZE;
    let viewport_h = VIEWPORT_BOARD_H as f32 * CELL_SIZE;
    let x = MARGIN + (viewport_w - board_w) / 2.0;
    let y = MARGIN + UI_STRIP + (viewport_h - board_h) / 2.0;
    (x, y)
}

pub fn centered_text(text: &str, y: f32, size: f32, color: Color) {
    let center_x = WINDOW_WIDTH as f32 / 2.0;
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, center_x - dims.width / 2.0, y, size, color);
}

pub fn draw_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::from_rgba(20, 20, 30, 230));
    draw_rectangle_lines(x, y, w, h, 2.0, Color::from_rgba(100, 100, 140, 255));
}
