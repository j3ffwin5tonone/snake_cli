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

// ---------------------------------------------------------------------------
// CRT Phosphor Terminal palette
// ---------------------------------------------------------------------------
// Two-phosphor look, like an old dual-color amber/green terminal monitor.
// Green is the primary ink; amber is reserved for highlights, rewards and
// warnings so it reads as a distinct "second phosphor" rather than a random
// accent.
pub const CRT_BG: Color = Color::new(0.016, 0.039, 0.020, 1.0); // #040a05
pub const CRT_BOARD_BG: Color = Color::new(0.008, 0.024, 0.016, 1.0); // #020604
pub const CRT_GREEN: Color = Color::new(0.239, 1.0, 0.431, 1.0); // #3dff6e - bright phosphor (head, key text)
pub const CRT_GREEN_MID: Color = Color::new(0.165, 0.604, 0.298, 1.0); // #2a9a4c - secondary text
pub const CRT_GREEN_DIM: Color = Color::new(0.165, 0.478, 0.247, 1.0); // #2a7a3f - borders / dim labels
pub const CRT_GREEN_BODY: Color = Color::new(0.110, 0.427, 0.200, 1.0); // #1c6d33 - snake body segments
pub const CRT_AMBER: Color = Color::new(1.0, 0.690, 0.0, 1.0); // #ffb000 - second phosphor: rewards/highlights
pub const CRT_CYAN: Color = Color::new(0.29, 0.88, 1.0, 1.0); // #4ae0ff - speed boost accent
pub const CRT_RED: Color = Color::new(0.753, 0.294, 0.294, 1.0); // #c04b4b - death cause / danger

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

// Draws text with a soft phosphor glow: a few low-alpha offset copies behind
// the crisp main draw. Cheap stand-in for a real bloom/blur pass.
pub fn glow_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let glow = Color::new(color.r, color.g, color.b, 0.18);
    for &(ox, oy) in &[(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
        draw_text(text, x + ox, y + oy, size, glow);
    }
    draw_text(text, x, y, size, color);
}

// glow_text, horizontally centered like centered_text.
pub fn centered_glow_text(text: &str, y: f32, size: f32, color: Color) {
    let center_x = WINDOW_WIDTH as f32 / 2.0;
    let dims = measure_text(text, None, size as u16, 1.0);
    glow_text(text, center_x - dims.width / 2.0, y, size, color);
}

pub fn draw_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::new(0.024, 0.078, 0.039, 0.55));
    draw_rectangle_lines(x, y, w, h, 1.0, CRT_GREEN_DIM);
}

// Bracket-style label breaking the top border, e.g. "[ MENU ]", terminal-window style.
pub fn draw_panel_label(x: f32, y: f32, label: &str) {
    let size = 16.0;
    let dims = measure_text(label, None, size as u16, 1.0);
    draw_rectangle(
        x + 10.0,
        y - size * 0.72,
        dims.width + 16.0,
        size * 0.9,
        CRT_BG,
    );
    draw_text(label, x + 18.0, y, size, CRT_GREEN);
}

// Full-screen scanline + vignette overlay. Call once per frame, after all
// other drawing, so it sits on top like glass.
pub fn draw_crt_overlay() {
    let w = WINDOW_WIDTH as f32;
    let h = WINDOW_HEIGHT as f32;
    let mut py = 0.0;
    while py < h {
        draw_rectangle(0.0, py, w, 2.0, Color::new(0.0, 0.0, 0.0, 0.28));
        py += 4.0;
    }
    let edge = Color::new(0.0, 0.0, 0.0, 0.35);
    draw_rectangle(0.0, 0.0, w, 24.0, edge);
    draw_rectangle(0.0, h - 24.0, w, 24.0, edge);
    draw_rectangle(0.0, 0.0, 24.0, h, edge);
    draw_rectangle(w - 24.0, 0.0, 24.0, h, edge);
}
