use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::game::{DeathCause, Game, GameMode, RunStats};

const LEADERBOARD_FILE: &str = ".snake_cli_leaderboard";
const SETTINGS_FILE: &str = ".snake_cli_settings";
const ACHIEVEMENTS_FILE: &str = ".snake_cli_achievements";
const MAX_ENTRIES: usize = 5;
pub const MAX_NAME_LEN: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreEntry {
    pub score: u16,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct ModeLeaderboard {
    entries: Vec<ScoreEntry>,
}

impl ModeLeaderboard {
    pub fn entries(&self) -> &[ScoreEntry] {
        &self.entries
    }

    pub fn top_score(&self) -> u16 {
        self.entries.first().map(|e| e.score).unwrap_or(0)
    }

    fn insert(&mut self, score: u16, name: &str) -> bool {
        if score == 0 {
            return false;
        }
        let name = sanitize_name(name);
        self.entries.push(ScoreEntry { score, name });
        self.entries.sort_by_key(|e| std::cmp::Reverse(e.score));
        self.entries.truncate(MAX_ENTRIES);
        self.entries.first().map(|e| e.score) == Some(score)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Leaderboards {
    pub classic: ModeLeaderboard,
    pub wrap: ModeLeaderboard,
    pub daily: ModeLeaderboard,
    pub daily_date_key: String,
}

impl Leaderboards {
    pub fn load() -> Self {
        let content = fs::read_to_string(leaderboard_path()).unwrap_or_default();
        let mut boards = Leaderboards {
            daily_date_key: Game::daily_date_key(),
            ..Default::default()
        };
        for line in content.lines() {
            if let Some(entry) = parse_line(line) {
                boards.apply_entry(entry);
            }
        }
        boards.migrate_legacy_if_needed();
        boards
    }

    fn migrate_legacy_if_needed(&mut self) {
        // Legacy format: score|name without mode prefix — already handled in parse_line
    }

    fn apply_entry(&mut self, entry: ParsedEntry) {
        if entry.mode == "daily" && entry.date_key.as_deref() != Some(self.daily_date_key.as_str())
        {
            return;
        }
        let board = match entry.mode.as_str() {
            "wrap" => &mut self.wrap,
            "daily" => &mut self.daily,
            _ => &mut self.classic,
        };
        board.entries.push(ScoreEntry {
            score: entry.score,
            name: entry.name,
        });
        board.entries.sort_by_key(|e| std::cmp::Reverse(e.score));
        board.entries.truncate(MAX_ENTRIES);
    }

    pub fn for_mode(&self, mode: GameMode) -> &ModeLeaderboard {
        match mode {
            GameMode::Classic => &self.classic,
            GameMode::Wrap => &self.wrap,
            GameMode::Daily => &self.daily,
        }
    }

    pub fn top_score_for(&self, mode: GameMode) -> u16 {
        self.for_mode(mode).top_score()
    }

    /// Returns true if this score is a new #1 for the mode.
    pub fn add_score(&mut self, mode: GameMode, score: u16, name: &str) -> bool {
        let is_new = self.for_mode_mut(mode).insert(score, name);
        self.persist();
        is_new
    }

    fn for_mode_mut(&mut self, mode: GameMode) -> &mut ModeLeaderboard {
        match mode {
            GameMode::Classic => &mut self.classic,
            GameMode::Wrap => &mut self.wrap,
            GameMode::Daily => &mut self.daily,
        }
    }

    fn persist(&self) {
        let mut lines = Vec::new();
        for entry in &self.classic.entries {
            lines.push(format!(
                "{}|{}|{}",
                GameMode::Classic.persist_key(),
                entry.score,
                entry.name
            ));
        }
        for entry in &self.wrap.entries {
            lines.push(format!(
                "{}|{}|{}",
                GameMode::Wrap.persist_key(),
                entry.score,
                entry.name
            ));
        }
        for entry in &self.daily.entries {
            lines.push(format!(
                "daily|{}|{}|{}",
                self.daily_date_key, entry.score, entry.name
            ));
        }
        if let Err(e) = fs::write(leaderboard_path(), lines.join("\n")) {
            #[cfg(debug_assertions)]
            eprintln!("Failed to write leaderboard: {e}");
        }
    }
}

struct ParsedEntry {
    mode: String,
    score: u16,
    name: String,
    date_key: Option<String>,
}

fn parse_line(line: &str) -> Option<ParsedEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split('|').collect();
    match parts.as_slice() {
        ["daily", date_key, score_str, name] => {
            let score: u16 = score_str.trim().parse().ok()?;
            Some(ParsedEntry {
                mode: "daily".to_string(),
                score,
                name: normalize_legacy_name(name.trim()),
                date_key: Some((*date_key).to_string()),
            })
        }
        [mode, score_str, name] => {
            let score: u16 = score_str.trim().parse().ok()?;
            Some(ParsedEntry {
                mode: (*mode).to_string(),
                score,
                name: normalize_legacy_name(name.trim()),
                date_key: None,
            })
        }
        [score_str, name] => {
            let score: u16 = score_str.trim().parse().ok()?;
            Some(ParsedEntry {
                mode: "classic".to_string(),
                score,
                name: normalize_legacy_name(name.trim()),
                date_key: None,
            })
        }
        _ => None,
    }
}

// Leaderboards replaces the old single-mode Leaderboard type.

fn settings_path() -> PathBuf {
    config_path(SETTINGS_FILE)
}

fn leaderboard_path() -> PathBuf {
    config_path(LEADERBOARD_FILE)
}

fn achievements_path() -> PathBuf {
    config_path(ACHIEVEMENTS_FILE)
}

fn config_path(file: &str) -> PathBuf {
    let mut path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path.push(file);
    path
}

pub fn load_sound_enabled() -> bool {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|s| {
            s.lines().find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "sound").then(|| value.trim() == "on" || value.trim() == "1")
            })
        })
        .unwrap_or(true)
}

pub fn save_sound_enabled(enabled: bool) {
    let value = if enabled { "on" } else { "off" };
    if let Err(e) = fs::write(settings_path(), format!("sound={value}\n")) {
        #[cfg(debug_assertions)]
        eprintln!("Failed to write settings: {e}");
    }
}

fn sanitize_name(name: &str) -> String {
    let trimmed: String = name
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .take(MAX_NAME_LEN)
        .collect();
    if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed
    }
}

fn normalize_legacy_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() >= 4 {
        "Player".to_string()
    } else {
        trimmed.to_string()
    }
}

// --- Achievements ---

pub struct AchievementDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

pub const ACHIEVEMENTS: &[AchievementDef] = &[
    AchievementDef {
        id: "first_bite",
        title: "First Bite",
        description: "Eat your first food",
    },
    AchievementDef {
        id: "double_digits",
        title: "Double Digits",
        description: "Score 10 or more",
    },
    AchievementDef {
        id: "speed_demon",
        title: "Speed Demon",
        description: "Reach level 5",
    },
    AchievementDef {
        id: "perfect_run",
        title: "Perfect Run",
        description: "Fill the entire board",
    },
    AchievementDef {
        id: "wrap_master",
        title: "Wrap Master",
        description: "Score 20 in Wrap mode",
    },
    AchievementDef {
        id: "combo_king",
        title: "Combo King",
        description: "Reach a 5x streak",
    },
    AchievementDef {
        id: "golden_hour",
        title: "Golden Hour",
        description: "Eat 3 golden apples in one run",
    },
    AchievementDef {
        id: "survivor",
        title: "Survivor",
        description: "Survive for 2 minutes",
    },
];

#[derive(Clone, Debug, Default)]
pub struct Achievements {
    unlocked: HashSet<String>,
    newly_unlocked: Vec<String>,
}

impl Achievements {
    pub fn load() -> Self {
        let unlocked = fs::read_to_string(achievements_path())
            .ok()
            .map(|content| {
                content
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Achievements {
            unlocked,
            newly_unlocked: Vec::new(),
        }
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    pub fn total_count(&self) -> usize {
        ACHIEVEMENTS.len()
    }

    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocked.contains(id)
    }

    pub fn take_newly_unlocked(&mut self) -> Vec<&'static AchievementDef> {
        let ids: Vec<String> = self.newly_unlocked.drain(..).collect();
        ids.iter()
            .filter_map(|id| ACHIEVEMENTS.iter().find(|a| a.id == id))
            .collect()
    }

    pub fn evaluate_run(&mut self, game: &Game, won: bool, death_cause: Option<DeathCause>) {
        let _ = death_cause;
        let stats: &RunStats = &game.stats;
        let checks: &[(&str, bool)] = &[
            ("first_bite", stats.food_eaten >= 1),
            ("double_digits", game.score >= 10),
            ("speed_demon", game.level() >= 5),
            ("perfect_run", won),
            (
                "wrap_master",
                game.mode == GameMode::Wrap && game.score >= 20,
            ),
            ("combo_king", stats.max_streak >= 5),
            ("golden_hour", stats.golden_eaten >= 3),
            ("survivor", game.duration_secs() >= 120.0),
        ];
        for (id, condition) in checks {
            if *condition {
                self.unlock(id);
            }
        }
    }

    fn unlock(&mut self, id: &str) {
        if self.unlocked.insert(id.to_string()) {
            self.newly_unlocked.push(id.to_string());
        }
    }

    pub fn persist(&self) {
        let content: Vec<&str> = self.unlocked.iter().map(String::as_str).collect();
        if let Err(e) = fs::write(achievements_path(), content.join("\n")) {
            #[cfg(debug_assertions)]
            eprintln!("Failed to write achievements: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_format() {
        let entry = parse_line("10|Alice").unwrap();
        assert_eq!(entry.mode, "classic");
        assert_eq!(entry.score, 10);
        assert_eq!(entry.name, "Alice");
    }

    #[test]
    fn parse_mode_format() {
        let entry = parse_line("wrap|8|Bob").unwrap();
        assert_eq!(entry.mode, "wrap");
        assert_eq!(entry.score, 8);
    }

    #[test]
    fn sanitize_name_defaults_when_empty() {
        assert_eq!(sanitize_name("   "), "Player");
        assert_eq!(sanitize_name("  Ada  "), "Ada");
    }

    #[test]
    fn normalize_legacy_day_numbers() {
        assert_eq!(normalize_legacy_name("20647"), "Player");
        assert_eq!(normalize_legacy_name("Andi"), "Andi");
    }

    #[test]
    fn sorts_entries_descending() {
        let mut board = ModeLeaderboard::default();
        for s in 1..=7 {
            board.insert(s, &format!("P{s}"));
        }
        assert_eq!(board.entries.len(), 5);
        assert_eq!(board.entries[0].score, 7);
        assert_eq!(board.entries[4].score, 3);
    }

    #[test]
    fn achievement_unlock_once() {
        let mut ach = Achievements::default();
        ach.unlock("first_bite");
        ach.unlock("first_bite");
        assert_eq!(ach.unlocked.len(), 1);
        assert_eq!(ach.newly_unlocked.len(), 1);
    }
}
