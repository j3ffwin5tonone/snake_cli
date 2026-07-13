use std::fs;
use std::path::PathBuf;

const LEADERBOARD_FILE: &str = ".snake_cli_leaderboard";
const MAX_ENTRIES: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreEntry {
    pub score: u16,
    pub date: String,
}

#[derive(Clone, Debug, Default)]
pub struct Leaderboard {
    entries: Vec<ScoreEntry>,
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
        let entries = fs::read_to_string(leaderboard_path())
            .ok()
            .map(|content| parse_entries(&content))
            .unwrap_or_default();
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
    pub fn add_score(&mut self, score: u16) -> bool {
        if score == 0 {
            return false;
        }

        let date = chrono_date();
        self.entries.push(ScoreEntry { score, date });
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
            let (score_str, date) = line.split_once('|')?;
            let score: u16 = score_str.trim().parse().ok()?;
            Some(ScoreEntry {
                score,
                date: date.trim().to_string(),
            })
        })
        .collect()
}

fn serialize_entries(entries: &[ScoreEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}|{}", e.score, e.date))
        .collect::<Vec<_>>()
        .join("\n")
}

fn chrono_date() -> String {
    // Simple date without adding a chrono dependency.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs / 86400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize_roundtrip() {
        let raw = "10|123\n5|456";
        let entries = parse_entries(raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].score, 10);
        assert_eq!(serialize_entries(&entries), raw);
    }

    #[test]
    fn sorts_entries_descending() {
        let mut entries: Vec<ScoreEntry> = (1..=7)
            .map(|s| ScoreEntry {
                score: s,
                date: s.to_string(),
            })
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.score));
        entries.truncate(MAX_ENTRIES);
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].score, 7);
        assert_eq!(entries[4].score, 3);
    }
}
