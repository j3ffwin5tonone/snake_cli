use macroquad::prelude::*;

use crate::game::{FoodKind, Game};

use super::effects::{draw_eat_flash, draw_level_up_edge};
use super::{board_offset, cell_size_for, centered_text};

pub fn draw_playing(game: &Game, visuals: &super::VisualState, tick_alpha: f32, top_score: u16) {
    let shake_x = visuals.shake_offset();
    let cell_size = cell_size_for(&game.config);
    let (board_x, board_y) = board_offset(&game.config, cell_size);
    let board_w = game.config.board_width as f32 * cell_size;
    let board_h = game.config.board_height as f32 * cell_size;

    clear_background(Color::from_rgba(18, 18, 24, 255));

    super::hud::draw_hud(game, top_score, shake_x);

    draw_rectangle(
        board_x + shake_x,
        board_y,
        board_w,
        board_h,
        Color::from_rgba(30, 30, 40, 255),
    );

    for x in 0..=game.config.board_width {
        let px = board_x + shake_x + x as f32 * cell_size;
        draw_line(
            px,
            board_y,
            px,
            board_y + board_h,
            1.0,
            Color::from_rgba(45, 45, 58, 255),
        );
    }
    for y in 0..=game.config.board_height {
        let py = board_y + y as f32 * cell_size;
        draw_line(
            board_x + shake_x,
            py,
            board_x + shake_x + board_w,
            py,
            1.0,
            Color::from_rgba(45, 45, 58, 255),
        );
    }

    draw_rectangle_lines(
        board_x + shake_x,
        board_y,
        board_w,
        board_h,
        2.0,
        Color::from_rgba(80, 80, 100, 255),
    );

    draw_eat_flash(
        board_x + shake_x,
        board_y,
        board_w,
        board_h,
        visuals.eat_flash,
    );
    draw_level_up_edge(
        board_x + shake_x,
        board_y,
        board_w,
        board_h,
        visuals.level_up_flash,
    );

    draw_food(game, board_x + shake_x, board_y, cell_size);
    draw_snake(
        game,
        visuals,
        tick_alpha,
        board_x + shake_x,
        board_y,
        cell_size,
    );

    if let Some(ate) = &game.last_ate
        && visuals.eat_flash > 0.5
    {
        let popup = format!("+{}", ate.points);
        let popup_color = match ate.kind {
            FoodKind::Golden => Color::from_rgba(255, 215, 0, 255),
            FoodKind::SpeedBoost => Color::from_rgba(100, 200, 255, 255),
            FoodKind::Normal => Color::from_rgba(255, 255, 100, 255),
        };
        centered_text(&popup, board_y + board_h / 2.0, 24.0, popup_color);
        if ate.streak >= 2 {
            centered_text(
                &format!("x{} streak!", ate.streak),
                board_y + board_h / 2.0 + 28.0,
                16.0,
                Color::from_rgba(255, 180, 80, 255),
            );
        }
    }
}

fn cell_to_pixel(board_x: f32, board_y: f32, x: f32, y: f32, cell_size: f32) -> (f32, f32) {
    (board_x + x * cell_size, board_y + y * cell_size)
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

fn draw_food(game: &Game, board_x: f32, board_y: f32, cell_size: f32) {
    let pulse = ((get_time() as f32 * 4.0).sin() * 0.5 + 0.5) * 4.0;
    let (fx, fy) = cell_to_pixel(
        board_x,
        board_y,
        game.food.position.0 as f32,
        game.food.position.1 as f32,
        cell_size,
    );
    let color = match game.food.kind {
        FoodKind::Normal => Color::from_rgba(230, 70, 70, 255),
        FoodKind::Golden => Color::from_rgba(255, 215, 0, 255),
        FoodKind::SpeedBoost => Color::from_rgba(70, 180, 255, 255),
    };
    draw_rectangle(
        fx + 3.0 - pulse / 2.0,
        fy + 3.0 - pulse / 2.0,
        cell_size - 6.0 + pulse,
        cell_size - 6.0 + pulse,
        color,
    );
}

fn draw_snake(
    game: &Game,
    visuals: &super::VisualState,
    tick_alpha: f32,
    board_x: f32,
    board_y: f32,
    cell_size: f32,
) {
    let head = game.snake.head();
    for (i, &current) in game.snake.body.iter().enumerate() {
        let prev = visuals.prev_body.get(i).copied();
        let (cx, cy) = lerp_cell(prev, current, tick_alpha);
        let (px, py) = cell_to_pixel(board_x, board_y, cx, cy, cell_size);
        let inset = 2.0;
        let size = cell_size - inset * 2.0;
        let color = if current == head {
            Color::from_rgba(120, 230, 120, 255)
        } else {
            Color::from_rgba(70, 180, 70, 255)
        };
        draw_rectangle(px + inset, py + inset, size, size, color);
    }
}
