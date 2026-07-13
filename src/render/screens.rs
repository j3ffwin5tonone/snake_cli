use macroquad::prelude::*;

use crate::game::{BoardPreset, DeathCause, Game, GameMode};
use crate::persist::{ACHIEVEMENTS, Achievements, Leaderboards};

use super::board::draw_playing;
use super::{
    CRT_AMBER, CRT_GREEN, CRT_GREEN_DIM, CRT_GREEN_MID, CRT_RED, WINDOW_HEIGHT, WINDOW_WIDTH,
    centered_glow_text, centered_text, draw_panel, draw_panel_label,
};

pub fn draw_start_menu(
    mode: GameMode,
    preset: BoardPreset,
    leaderboards: &Leaderboards,
    player_name: &str,
    name_editing: bool,
    sound_enabled: bool,
    achievements: &Achievements,
) {
    clear_background(super::CRT_BG);

    centered_glow_text("SNAKE_", 56.0, 48.0, CRT_GREEN);

    let panel_w = 460.0;
    let panel_x = WINDOW_WIDTH as f32 / 2.0 - panel_w / 2.0;
    let panel_top = 100.0;

    let mut content_h = 28.0;
    content_h += 18.0 + 30.0 + 14.0;
    content_h += 22.0 * 3.0 + 12.0;
    content_h += 20.0;
    if mode == GameMode::Daily {
        content_h += 20.0;
    }
    content_h += 18.0;
    if achievements.unlocked_count() > 0 {
        content_h += 18.0;
    }
    content_h += 14.0 + 28.0 + 24.0;

    let panel_h = content_h;
    draw_panel(panel_x, panel_top, panel_w, panel_h);
    draw_panel_label(panel_x, panel_top, "[ MENU ]");

    let mut y = panel_top + 28.0;

    let name_label = if name_editing {
        "> NAME_  [TAB] DONE"
    } else {
        "> NAME_  [TAB] EDIT"
    };
    centered_text(name_label, y, 15.0, CRT_GREEN_MID);
    y += 22.0;

    let display_name = if player_name.is_empty() {
        "_".to_string()
    } else {
        player_name.to_uppercase()
    };
    let cursor = if name_editing && (get_time() as f32 * 2.0).sin() > 0.0 {
        "_"
    } else {
        ""
    };
    let name_color = if name_editing {
        CRT_GREEN
    } else {
        CRT_GREEN_MID
    };
    centered_glow_text(&format!("{display_name}{cursor}"), y, 24.0, name_color);
    y += 36.0;

    centered_text(
        &format!("MODE.......{}  [M]", mode.label().to_uppercase()),
        y,
        19.0,
        CRT_GREEN,
    );
    y += 24.0;
    centered_text(
        &format!("BOARD......{}  [B]", preset.label().to_uppercase()),
        y,
        19.0,
        CRT_GREEN,
    );
    y += 24.0;

    let sound_label = if sound_enabled { "ON" } else { "OFF" };
    centered_text(
        &format!("SOUND......{sound_label}  [T]"),
        y,
        19.0,
        CRT_GREEN,
    );
    y += 32.0;

    let mode_board = leaderboards.for_mode(mode);
    centered_glow_text(
        &format!(
            "HI-SCORE ({}): {:04}",
            mode.label().to_uppercase(),
            mode_board.top_score()
        ),
        y,
        17.0,
        CRT_AMBER,
    );
    y += 22.0;

    if mode == GameMode::Daily {
        centered_text(
            &format!("TODAY'S SEED: {}", Game::daily_date_key()),
            y,
            15.0,
            CRT_AMBER,
        );
        y += 22.0;
    }

    centered_text(
        &format!(
            "ACHIEVEMENTS: {:02}/{:02} UNLOCKED",
            achievements.unlocked_count(),
            achievements.total_count()
        ),
        y,
        15.0,
        CRT_GREEN_MID,
    );
    y += 20.0;

    if achievements.unlocked_count() > 0
        && let Some(ach) = ACHIEVEMENTS.iter().find(|a| achievements.is_unlocked(a.id))
    {
        centered_text(
            &format!("> {}", ach.title.to_uppercase()),
            y,
            14.0,
            CRT_AMBER,
        );
        y += 20.0;
    }

    y += 6.0;
    centered_glow_text("[ENTER] START_", y, 24.0, CRT_GREEN);

    let footer_y = panel_top + panel_h + 18.0;
    centered_text("WASD / ARROWS TO MOVE", footer_y, 14.0, CRT_GREEN_DIM);
    centered_text("[P] PAUSE   [Q] QUIT", footer_y + 20.0, 14.0, CRT_GREEN_DIM);

    super::draw_crt_overlay();
}

pub fn draw_countdown(seconds: f32) {
    clear_background(super::CRT_BG);
    let display = if seconds <= 0.0 {
        "GO!".to_string()
    } else {
        format!("{}", seconds.ceil() as u32)
    };
    centered_glow_text(&display, WINDOW_HEIGHT as f32 / 2.0, 68.0, CRT_GREEN);
    super::draw_crt_overlay();
}

pub fn draw_paused(
    game: &Game,
    visuals: &super::VisualState,
    tick_alpha: f32,
    top_score: u16,
    sound_enabled: bool,
) {
    draw_playing(game, visuals, tick_alpha, top_score);
    draw_rectangle(
        0.0,
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    centered_glow_text("PAUSED", WINDOW_HEIGHT as f32 / 2.0 - 30.0, 44.0, CRT_GREEN);
    let sound_label = if sound_enabled { "ON" } else { "OFF" };
    centered_text(
        &format!("SOUND: {sound_label}  [T] TOGGLE"),
        WINDOW_HEIGHT as f32 / 2.0 + 20.0,
        20.0,
        CRT_GREEN_MID,
    );
    centered_text(
        "[P] RESUME",
        WINDOW_HEIGHT as f32 / 2.0 + 50.0,
        22.0,
        CRT_GREEN_MID,
    );
    super::draw_crt_overlay();
}

pub fn draw_end_screen(
    game: &Game,
    leaderboards: &Leaderboards,
    is_new_record: bool,
    won: bool,
    death_cause: Option<DeathCause>,
    previous_best: u16,
    new_achievements: &[&crate::persist::AchievementDef],
) {
    clear_background(super::CRT_BG);

    let title = if won { "VICTORY" } else { "GAME OVER" };
    let title_color = if won { CRT_AMBER } else { CRT_GREEN };
    centered_glow_text(title, 36.0, 36.0, title_color);

    let diff = game.score as i32 - previous_best as i32;
    let diff_str = if diff > 0 {
        format!(" (+{diff} VS BEST)")
    } else if diff < 0 {
        format!(" ({diff} VS BEST)")
    } else if previous_best > 0 {
        " (MATCHED BEST)".to_string()
    } else {
        String::new()
    };
    centered_text(
        &format!("FINAL SCORE: {:04}{}", game.score, diff_str),
        72.0,
        20.0,
        CRT_GREEN,
    );

    let mins = (game.duration_secs() / 60.0) as u32;
    let secs = (game.duration_secs() % 60.0) as u32;
    let stats_line = format!(
        "FOOD {} \u{00B7} MAX STREAK x{} \u{00B7} TIME {mins}:{secs:02}",
        game.stats.food_eaten, game.stats.max_streak,
    );
    centered_text(&stats_line, 98.0, 15.0, CRT_GREEN_MID);

    let mut y = 118.0;

    if let Some(cause) = death_cause {
        let cause_text = match cause {
            DeathCause::Wall => "> HIT A WALL",
            DeathCause::SelfCollision => "> BIT YOURSELF",
        };
        centered_text(cause_text, y, 14.0, CRT_RED);
        y += 22.0;
    }

    if is_new_record {
        centered_glow_text("NEW HIGH SCORE!", y, 20.0, CRT_AMBER);
        y += 28.0;
    }

    if !new_achievements.is_empty() {
        centered_text("ACHIEVEMENTS UNLOCKED:", y, 14.0, CRT_GREEN);
        y += 20.0;
        for ach in new_achievements.iter().take(3) {
            centered_text(
                &format!("> {}", ach.title.to_uppercase()),
                y,
                14.0,
                CRT_GREEN_MID,
            );
            y += 16.0;
            centered_text(ach.description, y, 12.0, CRT_GREEN_DIM);
            y += 16.0;
        }
        if new_achievements.len() > 3 {
            centered_text(
                &format!("+ {} MORE", new_achievements.len() - 3),
                y,
                13.0,
                CRT_GREEN_DIM,
            );
            y += 16.0;
        }
    }

    y += 10.0;

    let panel_w = 420.0;
    let panel_h = 168.0;
    let panel_x = WINDOW_WIDTH as f32 / 2.0 - panel_w / 2.0;
    let panel_top = y;
    draw_panel(panel_x, panel_top, panel_w, panel_h);
    draw_panel_label(panel_x, panel_top, "[ TOP SCORES ]");

    let header_y = panel_top + 24.0;
    centered_text(
        &format!("({})", game.mode.label().to_uppercase()),
        header_y,
        16.0,
        CRT_GREEN_DIM,
    );

    let col_y = panel_top + 46.0;
    let col_size = 14.0;
    draw_text("#", panel_x + 28.0, col_y, col_size, CRT_GREEN_DIM);
    draw_text("NAME", panel_x + 64.0, col_y, col_size, CRT_GREEN_DIM);
    let score_label = "SCORE";
    let score_label_dims = measure_text(score_label, None, col_size as u16, 1.0);
    draw_text(
        score_label,
        panel_x + panel_w - 24.0 - score_label_dims.width,
        col_y,
        col_size,
        CRT_GREEN_DIM,
    );

    let entries = leaderboards.for_mode(game.mode).entries();
    let first_row_y = panel_top + 68.0;
    let row_spacing = 22.0;

    if entries.is_empty() {
        centered_text("NO SCORES YET", first_row_y + 16.0, 17.0, CRT_GREEN_MID);
    } else {
        for (i, entry) in entries.iter().take(5).enumerate() {
            draw_leaderboard_row(
                i + 1,
                &entry.name,
                entry.score,
                first_row_y + i as f32 * row_spacing,
                panel_x,
                panel_w,
            );
        }
    }

    let footer_y = panel_top + panel_h + 14.0;
    centered_text("[ENTER] PLAY AGAIN", footer_y, 15.0, CRT_GREEN_MID);
    centered_text("[Q] QUIT", footer_y + 18.0, 15.0, CRT_GREEN_MID);

    super::draw_crt_overlay();
}

fn draw_leaderboard_row(rank: usize, name: &str, score: u16, y: f32, panel_x: f32, panel_w: f32) {
    let rank_text = format!("{rank}");
    let score_text = format!("{:04}", score);
    let row_size = 19.0;

    let rank_x = panel_x + 24.0;
    let name_x = panel_x + 64.0;
    let score_right = panel_x + panel_w - 24.0;

    draw_text(&rank_text, rank_x, y, row_size, CRT_GREEN);
    draw_text(name.to_uppercase(), name_x, y, row_size, CRT_GREEN);

    let score_dims = measure_text(&score_text, None, row_size as u16, 1.0);
    draw_text(
        &score_text,
        score_right - score_dims.width,
        y,
        row_size,
        CRT_AMBER,
    );
}
