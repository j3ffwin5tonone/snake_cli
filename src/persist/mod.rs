use std::fs;
use std::path::PathBuf;

const LEADERBOARD_FILE: &str = ".snake_cli_leaderboard";
const SETTINGS_FILE: &str = ".snake_cli_settings";
const MAX_ENTRIES: usize = 5;
pub const MAX_NAME_LEN: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreEntry {
    pub score: u16,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct Leaderboard {
    entries: Vec<ScoreEntry>,
}

fn settings_path() -> PathBuf {
    let mut path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path.push(SETTINGS_FILE);
    path
}

pub fn load_sound_enabled() -> bool {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|s| s.lines().find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "sound").then(|| value.trim() == "on" || value.trim() == "1")
        }))
        .unwrap_or(true)
}

pub fn save_sound_enabled(enabled: bool) {
    let value = if enabled { "on" } else { "off" };
    let _ = fs::write(settings_path(), format!("sound={value}\n"));
}

fn leaderboard_path() -> PathBuf {
    let mut path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path.push(LEADERBOARD_FILE);
    path
}

impl Leaderboard {
    pub fn load() -> Self {
        let mut entries = fs::read_to_string(leaderboard_path())
            .ok()
            .map(|content| parse_entries(&content))
            .unwrap_or_default();

        let migrated = migrate_legacy_entries(&mut entries);
        if migrated {
            let _ = fs::write(leaderboard_path(), serialize_entries(&entries));
        }

        Leaderboard { entries }
    }

    pub fn entries(&self) -> &[ScoreEntry] {
        &self.entries
    }

    pub fn top_score(&self) -> u16 {
        self.entries.first().map(|e| e.score).unwrap_or(0)
    }

    /// Insert a score, keep top 5 sorted descending, persist to disk.
    /// Returns true if this score is a new #1.
    pub fn add_score(&mut self, score: u16, name: &str) -> bool {
        if score == 0 {
            return false;
        }

        let name = sanitize_name(name);
        self.entries.push(ScoreEntry { score, name });
        self.entries.sort_by_key(|e| std::cmp::Reverse(e.score));
        self.entries.truncate(MAX_ENTRIES);

        let _ = fs::write(leaderboard_path(), serialize_entries(&self.entries));
        self.entries.first().map(|e| e.score) == Some(score)
    }
}

fn parse_entries(content: &str) -> Vec<ScoreEntry> {
    content
        .lines()
        .filter_map(|line| {
            let (score_str, name) = line.split_once('|')?;
            let score: u16 = score_str.trim().parse().ok()?;
            Some(ScoreEntry {
                score,
                name: name.trim().to_string(),
            })
        })
        .collect()
}

fn serialize_entries(entries: &[ScoreEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}|{}", e.score, e.name))
        .collect::<Vec<_>>()
        .join("\n")
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

/// Old leaderboard files stored a day-number instead of a player name.
fn normalize_legacy_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() >= 4 {
        "Player".to_string()
    } else {
        trimmed.to_string()
    }
}

fn migrate_legacy_entries(entries: &mut [ScoreEntry]) -> bool {
    let mut changed = false;
    for entry in entries.iter_mut() {
        let normalized = normalize_legacy_name(&entry.name);
        if normalized != entry.name {
            entry.name = normalized;
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize_roundtrip() {
        let raw = "10|Alice\n5|Bob";
        let entries = parse_entries(raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].score, 10);
        assert_eq!(entries[0].name, "Alice");
        assert_eq!(serialize_entries(&entries), raw);
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
    fn migrates_legacy_entries_on_load_format() {
        let mut entries = vec![ScoreEntry {
            score: 10,
            name: "20647".to_string(),
        }];
        assert!(migrate_legacy_entries(&mut entries));
        assert_eq!(entries[0].name, "Player");
    }

    #[test]
    fn sorts_entries_descending() {
        let mut entries: Vec<ScoreEntry> = (1..=7)
            .map(|s| ScoreEntry {
                score: s,
                name: format!("P{s}"),
            })
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.score));
        entries.truncate(MAX_ENTRIES);
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].score, 7);
        assert_eq!(entries[4].score, 3);
    }
}
