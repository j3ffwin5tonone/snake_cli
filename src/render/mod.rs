use macroquad::prelude::*;

use crate::game::{BOARD_HEIGHT, BOARD_WIDTH, Game, GameMode};
use crate::persist::Leaderboard;

pub const CELL_SIZE: f32 = 30.0;
pub const UI_STRIP: f32 = 50.0;
pub const MARGIN: f32 = 20.0;

pub const WINDOW_WIDTH: i32 = (BOARD_WIDTH as f32 * CELL_SIZE + MARGIN * 2.0) as i32;
pub const WINDOW_HEIGHT: i32 = (BOARD_HEIGHT as f32 * CELL_SIZE + UI_STRIP + MARGIN * 2.0) as i32;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

pub struct VisualState {
    pub prev_body: Vec<(u16, u16)>,
    pub eat_flash: f32,
    pub death_shake: f32,
}

impl Default for VisualState {
    fn default() -> Self {
        VisualState {
            prev_body: Vec::new(),
            eat_flash: 0.0,
            death_shake: 0.0,
        }
    }
}

impl VisualState {
    pub fn snapshot_body(&mut self, body: &[(u16, u16)]) {
        self.prev_body = body.to_vec();
    }

    pub fn trigger_eat(&mut self) {
        self.eat_flash = 1.0;
    }

    pub fn trigger_death(&mut self) {
        self.death_shake = 1.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.eat_flash = (self.eat_flash - dt * 3.0).max(0.0);
        self.death_shake = (self.death_shake - dt * 2.5).max(0.0);
    }
}

fn cell_to_pixel(x: f32, y: f32) -> (f32, f32) {
    let px = MARGIN + x * CELL_SIZE;
    let py = MARGIN + UI_STRIP + y * CELL_SIZE;
    (px, py)
}

fn lerp_cell(prev: Option<(u16, u16)>, current: (u16, u16), alpha: f32) -> (f32, f32) {
    match prev {
        Some((px, py)) => {
            let cx = px as f32 + (current.0 as f32 - px as f32) * alpha;
            let cy = py as f32 + (current.1 as f32 - py as f32) * alpha;
            (cx, cy)
        }
        None => (current.0 as f32, current.1 as f32),
    }
}

fn centered_text(text: &str, y: f32, size: f32, color: Color) {
    let center_x = WINDOW_WIDTH as f32 / 2.0;
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, center_x - dims.width / 2.0, y, size, color);
}

fn draw_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::from_rgba(20, 20, 30, 230));
    draw_rectangle_lines(x, y, w, h, 2.0, Color::from_rgba(100, 100, 140, 255));
}

pub fn draw_playing(game: &Game, visuals: &VisualState, tick_alpha: f32, top_score: u16) {
    let shake_x = if visuals.death_shake > 0.0 {
        (get_time() as f32 * 40.0).sin() * visuals.death_shake * 6.0
    } else {
        0.0
    };

    clear_background(Color::from_rgba(18, 18, 24, 255));

    let header = format!(
        "Score: {}  Level: {}  Highscore: {}",
        game.score,
        game.level(),
        top_score.max(game.score)
    );
    draw_text(&header, MARGIN + shake_x, MARGIN + 24.0, 28.0, WHITE);

    let mode_label = format!("Mode: {}", game.mode.label());
    draw_text(
        &mode_label,
        MARGIN + shake_x,
        MARGIN + 44.0,
        18.0,
        LIGHTGRAY,
    );

    let board_w = BOARD_WIDTH as f32 * CELL_SIZE;
    let board_h = BOARD_HEIGHT as f32 * CELL_SIZE;
    let board_x = MARGIN + shake_x;
    let board_y = MARGIN + UI_STRIP;

    draw_rectangle(
        board_x,
        board_y,
        board_w,
        board_h,
        Color::from_rgba(30, 30, 40, 255),
    );

    // Subtle grid.
    for x in 0..=BOARD_WIDTH {
        let px = board_x + x as f32 * CELL_SIZE;
        draw_line(
            px,
            board_y,
            px,
            board_y + board_h,
            1.0,
            Color::from_rgba(45, 45, 58, 255),
        );
    }
    for y in 0..=BOARD_HEIGHT {
        let py = board_y + y as f32 * CELL_SIZE;
        draw_line(
            board_x,
            py,
            board_x + board_w,
            py,
            1.0,
            Color::from_rgba(45, 45, 58, 255),
        );
    }

    draw_rectangle_lines(
        board_x,
        board_y,
        board_w,
        board_h,
        2.0,
        Color::from_rgba(80, 80, 100, 255),
    );

    // Eat flash overlay on board.
    if visuals.eat_flash > 0.0 {
        draw_rectangle(
            board_x,
            board_y,
            board_w,
            board_h,
            Color::from_rgba(255, 255, 200, (visuals.eat_flash * 80.0) as u8),
        );
    }

    // Pulsing food.
    let pulse = ((get_time() as f32 * 4.0).sin() * 0.5 + 0.5) * 4.0;
    let (fx, fy) = cell_to_pixel(game.food.position.0 as f32, game.food.position.1 as f32);
    draw_rectangle(
        fx + 3.0 - pulse / 2.0 + shake_x,
        fy + 3.0 - pulse / 2.0,
        CELL_SIZE - 6.0 + pulse,
        CELL_SIZE - 6.0 + pulse,
        Color::from_rgba(230, 70, 70, 255),
    );

    let head = game.snake.head();
    let alpha = tick_alpha;
    for (i, &current) in game.snake.body.iter().enumerate() {
        let prev = visuals.prev_body.get(i).copied();
        let (cx, cy) = lerp_cell(prev, current, alpha);
        let (px, py) = cell_to_pixel(cx, cy);
        let inset = 2.0;
        let size = CELL_SIZE - inset * 2.0;
        let color = if current == head {
            Color::from_rgba(120, 230, 120, 255)
        } else {
            Color::from_rgba(70, 180, 70, 255)
        };
        draw_rectangle(px + inset + shake_x, py + inset, size, size, color);
    }
}

pub fn draw_start_menu(mode: GameMode, leaderboard: &Leaderboard) {
    clear_background(Color::from_rgba(18, 18, 24, 255));

    centered_text("SNAKE", 120.0, 56.0, Color::from_rgba(120, 230, 120, 255));

    let panel_w = 420.0;
    let panel_h = 220.0;
    let panel_x = WINDOW_WIDTH as f32 / 2.0 - panel_w / 2.0;
    draw_panel(panel_x, 160.0, panel_w, panel_h);

    centered_text(
        &format!("Mode: {}  [M] toggle", mode.label()),
        210.0,
        24.0,
        WHITE,
    );
    centered_text(
        &format!("Highscore: {}", leaderboard.top_score()),
        250.0,
        22.0,
        LIGHTGRAY,
    );
    centered_text("[Enter] Start", 300.0, 26.0, WHITE);
    centered_text("WASD / Arrows — Move", 340.0, 20.0, LIGHTGRAY);
    centered_text("[P] Pause   [Q] Quit", 365.0, 20.0, LIGHTGRAY);
}

pub fn draw_countdown(seconds: f32) {
    clear_background(Color::from_rgba(18, 18, 24, 255));
    let display = if seconds <= 0.0 {
        "GO!"
    } else {
        &format!("{}", seconds.ceil() as u32)
    };
    centered_text(display, WINDOW_HEIGHT as f32 / 2.0, 72.0, WHITE);
}

pub fn draw_paused(game: &Game, visuals: &VisualState, tick_alpha: f32, top_score: u16) {
    draw_playing(game, visuals, tick_alpha, top_score);
    draw_rectangle(
        0.0,
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        Color::from_rgba(0, 0, 0, 140),
    );
    centered_text("PAUSED", WINDOW_HEIGHT as f32 / 2.0 - 20.0, 48.0, WHITE);
    centered_text(
        "[P] Resume",
        WINDOW_HEIGHT as f32 / 2.0 + 30.0,
        24.0,
        LIGHTGRAY,
    );
}

pub fn draw_game_over(game: &Game, leaderboard: &Leaderboard, is_new_record: bool) {
    clear_background(Color::from_rgba(18, 18, 24, 255));

    centered_text("GAME OVER", 80.0, 48.0, WHITE);
    centered_text(&format!("Final Score: {}", game.score), 130.0, 28.0, WHITE);

    if is_new_record {
        centered_text(
            "New Highscore!",
            170.0,
            28.0,
            Color::from_rgba(255, 215, 0, 255),
        );
    }

    let panel_w = 360.0;
    let panel_h = 160.0;
    let panel_x = WINDOW_WIDTH as f32 / 2.0 - panel_w / 2.0;
    draw_panel(panel_x, 210.0, panel_w, panel_h);

    centered_text("Top Scores", 240.0, 22.0, LIGHTGRAY);
    let entries = leaderboard.entries();
    if entries.is_empty() {
        centered_text("No scores yet", 275.0, 20.0, WHITE);
    } else {
        for (i, entry) in entries.iter().take(5).enumerate() {
            let line = format!("{}. {}  ({})", i + 1, entry.score, entry.date);
            centered_text(&line, 275.0 + i as f32 * 24.0, 20.0, WHITE);
        }
    }

    centered_text(
        "[Enter] Play Again    [Q] Quit",
        WINDOW_HEIGHT as f32 - 40.0,
        22.0,
        LIGHTGRAY,
    );
}
