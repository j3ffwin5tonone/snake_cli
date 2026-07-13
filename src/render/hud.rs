use macroquad::prelude::*;

use crate::game::Game;

use super::MARGIN;

pub fn draw_hud(game: &Game, top_score: u16, shake_x: f32) {
    let header = format!(
        "Score: {}  Level: {}  High: {}",
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

    if game.stats.current_streak >= 2 {
        let streak = format!("Streak: x{}", game.stats.current_streak);
        draw_text(
            &streak,
            MARGIN + shake_x + 280.0,
            MARGIN + 44.0,
            18.0,
            Color::from_rgba(255, 200, 80, 255),
        );
    }

    if game.speed_boost_ticks_remaining > 0 {
        draw_text(
            "BOOST!",
            MARGIN + shake_x + 400.0,
            MARGIN + 44.0,
            18.0,
            Color::from_rgba(70, 180, 255, 255),
        );
    }
}
