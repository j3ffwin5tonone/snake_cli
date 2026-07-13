use macroquad::prelude::*;

use crate::game::Game;

use super::{CRT_AMBER, CRT_CYAN, CRT_GREEN, CRT_GREEN_MID, MARGIN, glow_text};

pub fn draw_hud(game: &Game, top_score: u16, shake_x: f32) {
    let header = format!(
        "SCORE {:04}  LVL {:02}  HI {:04}",
        game.score,
        game.level(),
        top_score.max(game.score)
    );
    glow_text(&header, MARGIN + shake_x, MARGIN + 24.0, 26.0, CRT_GREEN);

    let mode_label = format!("MODE: {}", game.mode.label().to_uppercase());
    draw_text(
        &mode_label,
        MARGIN + shake_x,
        MARGIN + 46.0,
        17.0,
        CRT_GREEN_MID,
    );

    if game.stats.current_streak >= 2 {
        let streak = format!("STREAK x{}", game.stats.current_streak);
        glow_text(
            &streak,
            MARGIN + shake_x + 280.0,
            MARGIN + 46.0,
            17.0,
            CRT_AMBER,
        );
    }

    if game.speed_boost_ticks_remaining > 0 {
        glow_text(
            "BOOST!",
            MARGIN + shake_x + 420.0,
            MARGIN + 46.0,
            17.0,
            CRT_CYAN,
        );
    }
}
