use macroquad::prelude::*;

use crate::game::{BoardPreset, DeathCause, Game, GameMode};
use crate::persist::{ACHIEVEMENTS, Achievements, Leaderboards};

use super::board::draw_playing;
use super::{WINDOW_HEIGHT, WINDOW_WIDTH, centered_text, draw_panel};

pub fn draw_start_menu(
    mode: GameMode,
    preset: BoardPreset,
    leaderboards: &Leaderboards,
    player_name: &str,
    name_editing: bool,
    sound_enabled: bool,
    achievements: &Achievements,
) {
    clear_background(Color::from_rgba(18, 18, 24, 255));

    centered_text("SNAKE", 56.0, 52.0, Color::from_rgba(120, 230, 120, 255));

    let panel_w = 440.0;
    let panel_x = WINDOW_WIDTH as f32 / 2.0 - panel_w / 2.0;
    let panel_top = 100.0;

    // Measure content height first so the panel fits its contents.
    let mut content_h = 28.0; // top padding
    content_h += 18.0 + 30.0 + 14.0; // name label + input + gap
    content_h += 22.0 * 3.0 + 12.0; // mode, board, sound + gap
    content_h += 20.0; // highscore
    if mode == GameMode::Daily {
        content_h += 20.0;
    }
    content_h += 18.0; // achievements count
    if achievements.unlocked_count() > 0 {
        content_h += 18.0; // latest achievement
    }
    content_h += 14.0 + 28.0 + 24.0; // gap + start button + bottom padding

    let panel_h = content_h;
    draw_panel(panel_x, panel_top, panel_w, panel_h);

    let mut y = panel_top + 28.0;

    let name_label = if name_editing {
        "Your name  [Tab] done editing"
    } else {
        "Your name  [Tab] edit"
    };
    centered_text(name_label, y, 16.0, LIGHTGRAY);
    y += 22.0;

    let display_name = if player_name.is_empty() {
        "_".to_string()
    } else {
        player_name.to_string()
    };
    let cursor = if name_editing && (get_time() as f32 * 2.0).sin() > 0.0 {
        "|"
    } else {
        ""
    };
    let name_color = if name_editing {
        WHITE
    } else {
        Color::from_rgba(180, 180, 200, 255)
    };
    centered_text(&format!("{display_name}{cursor}"), y, 26.0, name_color);
    y += 36.0;

    centered_text(&format!("Mode: {}  [M]", mode.label()), y, 20.0, WHITE);
    y += 24.0;
    centered_text(&format!("Board: {}  [B]", preset.label()), y, 20.0, WHITE);
    y += 24.0;

    let sound_label = if sound_enabled { "On" } else { "Off" };
    centered_text(&format!("Sound: {sound_label}  [T]"), y, 20.0, WHITE);
    y += 32.0;

    let mode_board = leaderboards.for_mode(mode);
    centered_text(
        &format!("Highscore ({}): {}", mode.label(), mode_board.top_score()),
        y,
        18.0,
        LIGHTGRAY,
    );
    y += 22.0;

    if mode == GameMode::Daily {
        centered_text(
            &format!("Today's seed: {}", Game::daily_date_key()),
            y,
            16.0,
            Color::from_rgba(255, 200, 80, 255),
        );
        y += 22.0;
    }

    centered_text(
        &format!(
            "Achievements: {}/{} unlocked",
            achievements.unlocked_count(),
            achievements.total_count()
        ),
        y,
        16.0,
        LIGHTGRAY,
    );
    y += 20.0;

    if achievements.unlocked_count() > 0
        && let Some(ach) = ACHIEVEMENTS.iter().find(|a| achievements.is_unlocked(a.id))
    {
        centered_text(
            &format!("Latest: {}", ach.title),
            y,
            15.0,
            Color::from_rgba(120, 230, 120, 220),
        );
        y += 20.0;
    }

    y += 6.0;
    centered_text(
        "[Enter] Start",
        y,
        26.0,
        Color::from_rgba(120, 230, 120, 255),
    );

    let footer_y = panel_top + panel_h + 18.0;
    centered_text(
        "WASD / Arrows to move",
        footer_y,
        15.0,
        Color::from_rgba(140, 140, 160, 255),
    );
    centered_text(
        "[P] Pause   [Q] Quit",
        footer_y + 20.0,
        15.0,
        Color::from_rgba(140, 140, 160, 255),
    );
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
        Color::from_rgba(0, 0, 0, 140),
    );
    centered_text("PAUSED", WINDOW_HEIGHT as f32 / 2.0 - 30.0, 48.0, WHITE);
    let sound_label = if sound_enabled { "On" } else { "Off" };
    centered_text(
        &format!("Sound: {sound_label}  [T] toggle"),
        WINDOW_HEIGHT as f32 / 2.0 + 20.0,
        22.0,
        LIGHTGRAY,
    );
    centered_text(
        "[P] Resume",
        WINDOW_HEIGHT as f32 / 2.0 + 50.0,
        24.0,
        LIGHTGRAY,
    );
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
    clear_background(Color::from_rgba(18, 18, 24, 255));

    let title = if won { "VICTORY!" } else { "GAME OVER" };
    let title_color = if won {
        Color::from_rgba(255, 215, 0, 255)
    } else {
        WHITE
    };
    centered_text(title, 36.0, 40.0, title_color);

    let diff = game.score as i32 - previous_best as i32;
    let diff_str = if diff > 0 {
        format!(" (+{diff} vs best)")
    } else if diff < 0 {
        format!(" ({diff} vs best)")
    } else if previous_best > 0 {
        " (matched best)".to_string()
    } else {
        String::new()
    };
    centered_text(
        &format!("Final Score: {}{}", game.score, diff_str),
        72.0,
        22.0,
        WHITE,
    );

    let mins = (game.duration_secs() / 60.0) as u32;
    let secs = (game.duration_secs() % 60.0) as u32;
    let stats_line = format!(
        "Food: {} · Max streak: x{} · Time: {mins}:{secs:02}",
        game.stats.food_eaten, game.stats.max_streak,
    );
    centered_text(&stats_line, 98.0, 16.0, LIGHTGRAY);

    let mut y = 118.0;

    if let Some(cause) = death_cause {
        let cause_text = match cause {
            DeathCause::Wall => "Hit a wall",
            DeathCause::SelfCollision => "Bit yourself",
        };
        centered_text(cause_text, y, 15.0, Color::from_rgba(180, 100, 100, 255));
        y += 22.0;
    }

    if is_new_record {
        centered_text(
            "New Highscore!",
            y,
            22.0,
            Color::from_rgba(255, 215, 0, 255),
        );
        y += 28.0;
    }

    if !new_achievements.is_empty() {
        centered_text(
            "Achievements unlocked:",
            y,
            15.0,
            Color::from_rgba(120, 230, 120, 255),
        );
        y += 20.0;
        for ach in new_achievements.iter().take(3) {
            centered_text(
                &format!("• {}", ach.title),
                y,
                15.0,
                Color::from_rgba(160, 220, 160, 255),
            );
            y += 16.0;
            centered_text(
                ach.description,
                y,
                13.0,
                Color::from_rgba(130, 170, 130, 255),
            );
            y += 16.0;
        }
        if new_achievements.len() > 3 {
            centered_text(
                &format!("• +{} more", new_achievements.len() - 3),
                y,
                14.0,
                LIGHTGRAY,
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

    let header_y = panel_top + 24.0;
    centered_text(
        &format!("TOP SCORES ({})", game.mode.label()),
        header_y,
        18.0,
        LIGHTGRAY,
    );

    let col_y = panel_top + 46.0;
    let col_size = 15.0;
    draw_text(
        "#",
        panel_x + 28.0,
        col_y,
        col_size,
        Color::from_rgba(140, 140, 160, 255),
    );
    draw_text(
        "NAME",
        panel_x + 64.0,
        col_y,
        col_size,
        Color::from_rgba(140, 140, 160, 255),
    );
    let score_label = "SCORE";
    let score_label_dims = measure_text(score_label, None, col_size as u16, 1.0);
    draw_text(
        score_label,
        panel_x + panel_w - 24.0 - score_label_dims.width,
        col_y,
        col_size,
        Color::from_rgba(140, 140, 160, 255),
    );

    let entries = leaderboards.for_mode(game.mode).entries();
    let first_row_y = panel_top + 68.0;
    let row_spacing = 22.0;

    if entries.is_empty() {
        centered_text("No scores yet", first_row_y + 16.0, 18.0, WHITE);
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
    centered_text("[Enter] Play Again", footer_y, 16.0, LIGHTGRAY);
    centered_text("[Q] Quit", footer_y + 18.0, 16.0, LIGHTGRAY);
}

fn draw_leaderboard_row(rank: usize, name: &str, score: u16, y: f32, panel_x: f32, panel_w: f32) {
    let rank_text = format!("{rank}.");
    let score_text = score.to_string();
    let row_size = 20.0;

    let rank_x = panel_x + 24.0;
    let name_x = panel_x + 64.0;
    let score_right = panel_x + panel_w - 24.0;

    draw_text(&rank_text, rank_x, y, row_size, WHITE);
    draw_text(name, name_x, y, row_size, WHITE);

    let score_dims = measure_text(&score_text, None, row_size as u16, 1.0);
    draw_text(
        &score_text,
        score_right - score_dims.width,
        y,
        row_size,
        Color::from_rgba(120, 230, 120, 255),
    );
}
