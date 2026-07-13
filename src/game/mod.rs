use std::collections::VecDeque;

use ::rand::RngExt;
use ::rand::rngs::StdRng;
use ::rand::{SeedableRng, rng};

pub const BOARD_WIDTH: u16 = 20;
pub const BOARD_HEIGHT: u16 = 10;
pub const BASE_TICK_RATE_MS: u64 = 150;
pub const MIN_TICK_RATE_MS: u64 = 60;
pub const SPEED_RAMP_INTERVAL: u16 = 5;
pub const TICK_MS_DECREASE: u64 = 10;
pub const INPUT_BUFFER_SIZE: usize = 2;
pub const STREAK_WINDOW_TICKS: u16 = 8;
pub const SPEED_BOOST_DURATION_TICKS: u16 = 20;
pub const SPEED_BOOST_TICK_MS: u64 = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardPreset {
    Classic,
    Compact,
    Arena,
}

impl BoardPreset {
    pub fn toggle(self) -> Self {
        match self {
            BoardPreset::Classic => BoardPreset::Compact,
            BoardPreset::Compact => BoardPreset::Arena,
            BoardPreset::Arena => BoardPreset::Classic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BoardPreset::Classic => "Classic",
            BoardPreset::Compact => "Compact",
            BoardPreset::Arena => "Arena",
        }
    }

    pub fn config(self) -> GameConfig {
        match self {
            BoardPreset::Classic => GameConfig::default(),
            BoardPreset::Compact => GameConfig {
                board_width: 15,
                board_height: 8,
                ..GameConfig::default()
            },
            BoardPreset::Arena => GameConfig {
                board_width: 25,
                board_height: 15,
                ..GameConfig::default()
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub board_width: u16,
    pub board_height: u16,
    pub base_tick_ms: u64,
    pub min_tick_ms: u64,
    pub speed_ramp_interval: u16,
    pub tick_ms_decrease: u64,
    pub food_weight_normal: u8,
    pub food_weight_golden: u8,
    pub food_weight_speed_boost: u8,
    pub streak_window_ticks: u16,
    pub fixed_tick_rate: Option<u64>,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            board_width: BOARD_WIDTH,
            board_height: BOARD_HEIGHT,
            base_tick_ms: BASE_TICK_RATE_MS,
            min_tick_ms: MIN_TICK_RATE_MS,
            speed_ramp_interval: SPEED_RAMP_INTERVAL,
            tick_ms_decrease: TICK_MS_DECREASE,
            food_weight_normal: 85,
            food_weight_golden: 10,
            food_weight_speed_boost: 5,
            streak_window_ticks: STREAK_WINDOW_TICKS,
            fixed_tick_rate: None,
        }
    }
}

impl GameConfig {
    pub fn daily() -> Self {
        GameConfig {
            fixed_tick_rate: Some(120),
            speed_ramp_interval: u16::MAX,
            ..GameConfig::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Classic,
    Wrap,
    Daily,
}

impl GameMode {
    pub fn toggle(self) -> Self {
        match self {
            GameMode::Classic => GameMode::Wrap,
            GameMode::Wrap => GameMode::Daily,
            GameMode::Daily => GameMode::Classic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GameMode::Classic => "Classic",
            GameMode::Wrap => "Wrap",
            GameMode::Daily => "Daily",
        }
    }

    pub fn persist_key(self) -> &'static str {
        match self {
            GameMode::Classic => "classic",
            GameMode::Wrap => "wrap",
            GameMode::Daily => "daily",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodKind {
    Normal,
    Golden,
    SpeedBoost,
}

impl FoodKind {
    pub fn base_points(self) -> u16 {
        match self {
            FoodKind::Normal => 1,
            FoodKind::Golden => 5,
            FoodKind::SpeedBoost => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    Wall,
    SelfCollision,
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub body: Vec<(u16, u16)>,
    pub direction: Direction,
}

impl Snake {
    pub fn new(start_x: u16, start_y: u16) -> Self {
        Snake {
            body: vec![(start_x, start_y), (start_x.saturating_sub(1), start_y)],
            direction: Direction::Right,
        }
    }

    pub fn head(&self) -> (u16, u16) {
        *self.body.first().unwrap()
    }

    pub fn next_head_coords(&self) -> (i32, i32) {
        let (hx, hy) = self.head();
        match self.direction {
            Direction::Up => (hx as i32, hy as i32 - 1),
            Direction::Down => (hx as i32, hy as i32 + 1),
            Direction::Left => (hx as i32 - 1, hy as i32),
            Direction::Right => (hx as i32 + 1, hy as i32),
        }
    }

    pub fn resolve_head(
        coords: (i32, i32),
        mode: GameMode,
        board_width: u16,
        board_height: u16,
    ) -> Option<(u16, u16)> {
        let (x, y) = coords;
        match mode {
            GameMode::Classic | GameMode::Daily => {
                if x < 0 || y < 0 || x >= board_width as i32 || y >= board_height as i32 {
                    None
                } else {
                    Some((x as u16, y as u16))
                }
            }
            GameMode::Wrap => {
                let wx = ((x % board_width as i32) + board_width as i32) % board_width as i32;
                let wy = ((y % board_height as i32) + board_height as i32) % board_height as i32;
                Some((wx as u16, wy as u16))
            }
        }
    }

    pub fn advance_to(&mut self, new_head: (u16, u16), grew: bool) {
        self.body.insert(0, new_head);
        if !grew {
            self.body.pop();
        }
    }

    pub fn change_direction(&mut self, new_dir: Direction) {
        if new_dir != self.direction.opposite() {
            self.direction = new_dir;
        }
    }

    pub fn check_self_collision(&self) -> bool {
        let head = self.head();
        self.body.iter().skip(1).any(|&segment| segment == head)
    }
}

#[derive(Debug, Clone)]
pub struct Food {
    pub position: (u16, u16),
    pub kind: FoodKind,
}

impl Food {
    pub fn new(snake_body: &[(u16, u16)], config: &GameConfig, rng: &mut StdRng) -> Self {
        let mut food = Food {
            position: (0, 0),
            kind: FoodKind::Normal,
        };
        food.respawn(snake_body, config, rng);
        food
    }

    pub fn respawn(&mut self, snake_body: &[(u16, u16)], config: &GameConfig, rng: &mut StdRng) {
        let empty = empty_cells(snake_body, config);
        if empty.is_empty() {
            return;
        }
        let idx = rng.random_range(0..empty.len());
        self.position = empty[idx];
        self.kind = roll_food_kind(config, rng);
    }
}

fn empty_cells(snake_body: &[(u16, u16)], config: &GameConfig) -> Vec<(u16, u16)> {
    let mut cells = Vec::new();
    for y in 0..config.board_height {
        for x in 0..config.board_width {
            let pos = (x, y);
            if !snake_body.contains(&pos) {
                cells.push(pos);
            }
        }
    }
    cells
}

fn roll_food_kind(config: &GameConfig, rng: &mut StdRng) -> FoodKind {
    let total = config.food_weight_normal as u16
        + config.food_weight_golden as u16
        + config.food_weight_speed_boost as u16;
    let roll: u16 = rng.random_range(0..total);
    let golden_end = config.food_weight_normal as u16 + config.food_weight_golden as u16;
    if roll < config.food_weight_normal as u16 {
        FoodKind::Normal
    } else if roll < golden_end {
        FoodKind::Golden
    } else {
        FoodKind::SpeedBoost
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunStats {
    pub food_eaten: u16,
    pub max_streak: u16,
    pub current_streak: u16,
    pub golden_eaten: u16,
    pub ticks_survived: u64,
    pub ticks_since_last_eat: u16,
}

#[derive(Debug, Clone)]
pub struct AteInfo {
    pub kind: FoodKind,
    pub points: u16,
    pub streak: u16,
    pub leveled_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    Moved,
    Ate,
    Died(DeathCause),
    Won,
}

#[derive(Debug)]
pub struct Game {
    pub config: GameConfig,
    pub snake: Snake,
    pub food: Food,
    pub score: u16,
    pub mode: GameMode,
    pub pending_turns: VecDeque<Direction>,
    pub stats: RunStats,
    pub speed_boost_ticks_remaining: u16,
    pub last_ate: Option<AteInfo>,
    rng: StdRng,
}

impl Game {
    pub fn new(mode: GameMode, config: GameConfig) -> Self {
        Self::new_with_seed(mode, config, None)
    }

    pub fn new_with_seed(mode: GameMode, config: GameConfig, seed: Option<u64>) -> Self {
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => {
                let mut fallback = rng();
                StdRng::seed_from_u64(fallback.random())
            }
        };
        let start_x = config.board_width / 4;
        let start_y = config.board_height / 2;
        let snake = Snake::new(start_x, start_y);
        let food = Food::new(&snake.body, &config, &mut rng);

        Game {
            config,
            snake,
            food,
            score: 0,
            mode,
            pending_turns: VecDeque::new(),
            stats: RunStats::default(),
            speed_boost_ticks_remaining: 0,
            last_ate: None,
            rng,
        }
    }

    pub fn daily_seed_for_today() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        secs / 86_400
    }

    pub fn daily_date_key() -> String {
        format!("day{}", Self::daily_seed_for_today())
    }

    pub fn duration_secs(&self) -> f32 {
        self.stats.ticks_survived as f32 * self.config.base_tick_ms as f32 / 1000.0
    }

    pub fn level(&self) -> u16 {
        self.score / self.config.speed_ramp_interval + 1
    }

    pub fn tick_rate_ms(&self) -> u64 {
        if let Some(fixed) = self.config.fixed_tick_rate {
            return fixed;
        }
        if self.speed_boost_ticks_remaining > 0 {
            return SPEED_BOOST_TICK_MS;
        }
        let steps = (self.score / self.config.speed_ramp_interval) as u64;
        self.config
            .base_tick_ms
            .saturating_sub(steps * self.config.tick_ms_decrease)
            .max(self.config.min_tick_ms)
    }

    pub fn tick_seconds(&self) -> f32 {
        self.tick_rate_ms() as f32 / 1000.0
    }

    pub fn queue_turn(&mut self, dir: Direction) {
        let reference = self
            .pending_turns
            .back()
            .copied()
            .unwrap_or(self.snake.direction);
        if dir == reference.opposite() {
            return;
        }
        if self.pending_turns.len() >= INPUT_BUFFER_SIZE {
            return;
        }
        if self.pending_turns.back() == Some(&dir) {
            return;
        }
        self.pending_turns.push_back(dir);
    }

    fn apply_pending_turn(&mut self) {
        if let Some(dir) = self.pending_turns.pop_front() {
            self.snake.change_direction(dir);
        }
    }

    pub fn update(&mut self) -> UpdateResult {
        self.apply_pending_turn();
        self.stats.ticks_survived += 1;
        self.stats.ticks_since_last_eat = self.stats.ticks_since_last_eat.saturating_add(1);

        if self.speed_boost_ticks_remaining > 0 {
            self.speed_boost_ticks_remaining -= 1;
        }

        let next_coords = self.snake.next_head_coords();
        let Some(next_head) = Snake::resolve_head(
            next_coords,
            self.mode,
            self.config.board_width,
            self.config.board_height,
        ) else {
            return UpdateResult::Died(DeathCause::Wall);
        };

        let will_eat = next_head == self.food.position;
        self.snake.advance_to(next_head, will_eat);

        if self.snake.check_self_collision() {
            return UpdateResult::Died(DeathCause::SelfCollision);
        }

        if will_eat {
            return self.handle_eat();
        }

        if self.stats.ticks_since_last_eat > self.config.streak_window_ticks {
            self.stats.current_streak = 0;
        }

        UpdateResult::Moved
    }

    fn handle_eat(&mut self) -> UpdateResult {
        let kind = self.food.kind;
        let prev_score = self.score;

        if self.stats.ticks_since_last_eat <= self.config.streak_window_ticks
            && self.stats.current_streak > 0
        {
            self.stats.current_streak += 1;
        } else {
            self.stats.current_streak = 1;
        }
        self.stats.ticks_since_last_eat = 0;
        self.stats.max_streak = self.stats.max_streak.max(self.stats.current_streak);
        self.stats.food_eaten += 1;
        if kind == FoodKind::Golden {
            self.stats.golden_eaten += 1;
        }

        let streak_bonus = self.stats.current_streak / 3;
        let points = kind.base_points() + streak_bonus;
        self.score = self.score.saturating_add(points);

        if kind == FoodKind::SpeedBoost {
            self.speed_boost_ticks_remaining = SPEED_BOOST_DURATION_TICKS;
        }

        let leveled_up = self.score > 0
            && self.score.is_multiple_of(self.config.speed_ramp_interval)
            && self.score != prev_score;

        let ate_info = AteInfo {
            kind,
            points,
            streak: self.stats.current_streak,
            leveled_up,
        };
        self.last_ate = Some(ate_info);

        let empty = empty_cells(&self.snake.body, &self.config);
        if empty.is_empty() {
            return UpdateResult::Won;
        }

        self.food
            .respawn(&self.snake.body, &self.config, &mut self.rng);
        UpdateResult::Ate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> GameConfig {
        GameConfig {
            food_weight_normal: 100,
            food_weight_golden: 0,
            food_weight_speed_boost: 0,
            ..GameConfig::default()
        }
    }

    #[test]
    fn prevents_direction_reversal() {
        let mut snake = Snake::new(5, 5);
        snake.direction = Direction::Right;
        snake.change_direction(Direction::Left);
        assert_eq!(snake.direction, Direction::Right);
        snake.change_direction(Direction::Up);
        assert_eq!(snake.direction, Direction::Up);
    }

    #[test]
    fn queue_turn_buffers_and_blocks_reverse() {
        let mut game = Game::new_with_seed(GameMode::Classic, test_config(), Some(42));
        game.queue_turn(Direction::Left);
        assert_eq!(game.pending_turns.len(), 0);
        game.queue_turn(Direction::Up);
        assert_eq!(game.pending_turns.len(), 1);
        game.queue_turn(Direction::Down);
        assert_eq!(game.pending_turns.len(), 1);
        game.queue_turn(Direction::Right);
        assert_eq!(game.pending_turns.len(), 2);
    }

    #[test]
    fn grows_when_eating() {
        let mut game = Game::new_with_seed(GameMode::Classic, test_config(), Some(42));
        let len_before = game.snake.body.len();
        let next = Snake::resolve_head(
            game.snake.next_head_coords(),
            GameMode::Classic,
            game.config.board_width,
            game.config.board_height,
        )
        .unwrap();
        game.food.position = next;
        game.food.kind = FoodKind::Normal;
        game.update();
        assert_eq!(game.snake.body.len(), len_before + 1);
        assert_eq!(game.score, 1);
    }

    #[test]
    fn classic_wall_collision_ends_game() {
        let mut game = Game::new_with_seed(GameMode::Classic, test_config(), Some(42));
        game.snake.body = vec![(0, 0)];
        game.snake.direction = Direction::Left;
        assert!(matches!(
            game.update(),
            UpdateResult::Died(DeathCause::Wall)
        ));
    }

    #[test]
    fn wrap_mode_wraps_horizontally() {
        let mut snake = Snake::new(0, 5);
        snake.body = vec![(0, 5)];
        snake.direction = Direction::Left;
        let wrapped = Snake::resolve_head(
            snake.next_head_coords(),
            GameMode::Wrap,
            BOARD_WIDTH,
            BOARD_HEIGHT,
        )
        .unwrap();
        assert_eq!(wrapped, (BOARD_WIDTH - 1, 5));
    }

    #[test]
    fn speed_increases_every_five_food() {
        let mut game = Game::new_with_seed(GameMode::Classic, test_config(), Some(42));
        assert_eq!(game.tick_rate_ms(), BASE_TICK_RATE_MS);
        game.score = 5;
        assert!(game.tick_rate_ms() < BASE_TICK_RATE_MS);
        game.score = 100;
        assert_eq!(game.tick_rate_ms(), MIN_TICK_RATE_MS);
    }

    #[test]
    fn game_new_without_seed() {
        let game = Game::new(GameMode::Classic, GameConfig::default());
        assert_eq!(game.score, 0);
    }

    #[test]
    fn board_full_triggers_win() {
        let config = GameConfig {
            board_width: 3,
            board_height: 2,
            food_weight_normal: 100,
            food_weight_golden: 0,
            food_weight_speed_boost: 0,
            ..GameConfig::default()
        };
        let mut game = Game::new_with_seed(GameMode::Classic, config, Some(99));
        game.snake.body = vec![(1, 1), (0, 1), (0, 0), (1, 0), (2, 0)];
        game.snake.direction = Direction::Right;
        game.food.position = (2, 1);
        game.food.kind = FoodKind::Normal;
        assert!(matches!(game.update(), UpdateResult::Won));
    }

    #[test]
    fn streak_bonus_adds_points() {
        let mut game = Game::new_with_seed(GameMode::Classic, test_config(), Some(42));
        game.stats.current_streak = 3;
        game.stats.ticks_since_last_eat = 1;
        let next = Snake::resolve_head(
            game.snake.next_head_coords(),
            GameMode::Classic,
            game.config.board_width,
            game.config.board_height,
        )
        .unwrap();
        game.food.position = next;
        game.food.kind = FoodKind::Normal;
        game.update();
        assert_eq!(game.stats.current_streak, 4);
        assert!(game.score >= 2);
    }
}
