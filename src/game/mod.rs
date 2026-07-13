use ::rand::RngExt;
use ::rand::rng;

pub const BOARD_WIDTH: u16 = 20;
pub const BOARD_HEIGHT: u16 = 10;
pub const BASE_TICK_RATE_MS: u64 = 150;
pub const MIN_TICK_RATE_MS: u64 = 60;
pub const SPEED_RAMP_INTERVAL: u16 = 5;
pub const TICK_MS_DECREASE: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Classic,
    Wrap,
}

impl GameMode {
    pub fn toggle(self) -> Self {
        match self {
            GameMode::Classic => GameMode::Wrap,
            GameMode::Wrap => GameMode::Classic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GameMode::Classic => "Classic",
            GameMode::Wrap => "Wrap",
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

    pub fn resolve_head(coords: (i32, i32), mode: GameMode) -> Option<(u16, u16)> {
        let (x, y) = coords;
        match mode {
            GameMode::Classic => {
                if x < 0 || y < 0 || x >= BOARD_WIDTH as i32 || y >= BOARD_HEIGHT as i32 {
                    None
                } else {
                    Some((x as u16, y as u16))
                }
            }
            GameMode::Wrap => {
                let wx = ((x % BOARD_WIDTH as i32) + BOARD_WIDTH as i32) % BOARD_WIDTH as i32;
                let wy = ((y % BOARD_HEIGHT as i32) + BOARD_HEIGHT as i32) % BOARD_HEIGHT as i32;
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
}

impl Food {
    pub fn new(snake_body: &[(u16, u16)]) -> Self {
        let mut food = Food { position: (0, 0) };
        food.spawn(snake_body);
        food
    }

    pub fn spawn(&mut self, snake_body: &[(u16, u16)]) {
        let mut rng = rng();
        loop {
            let x: u16 = rng.random_range(0..BOARD_WIDTH);
            let y: u16 = rng.random_range(0..BOARD_HEIGHT);
            if !snake_body.contains(&(x, y)) {
                self.position = (x, y);
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub snake: Snake,
    pub food: Food,
    pub score: u16,
    pub mode: GameMode,
    pub game_over: bool,
    pub just_ate: bool,
}

impl Game {
    pub fn new(mode: GameMode) -> Self {
        let start_x = 5;
        let start_y = BOARD_HEIGHT / 2;
        let snake = Snake::new(start_x, start_y);
        let food = Food::new(&snake.body);

        Game {
            snake,
            food,
            score: 0,
            mode,
            game_over: false,
            just_ate: false,
        }
    }

    pub fn level(&self) -> u16 {
        self.score / SPEED_RAMP_INTERVAL + 1
    }

    pub fn tick_rate_ms(&self) -> u64 {
        let steps = (self.score / SPEED_RAMP_INTERVAL) as u64;
        BASE_TICK_RATE_MS
            .saturating_sub(steps * TICK_MS_DECREASE)
            .max(MIN_TICK_RATE_MS)
    }

    pub fn tick_seconds(&self) -> f32 {
        self.tick_rate_ms() as f32 / 1000.0
    }

    pub fn update(&mut self) -> UpdateResult {
        if self.game_over {
            return UpdateResult::None;
        }

        self.just_ate = false;
        let next_coords = self.snake.next_head_coords();
        let Some(next_head) = Snake::resolve_head(next_coords, self.mode) else {
            self.game_over = true;
            return UpdateResult::Died;
        };

        let will_eat = next_head == self.food.position;
        self.snake.advance_to(next_head, will_eat);

        if self.snake.check_self_collision() {
            self.game_over = true;
            return UpdateResult::Died;
        }

        if will_eat {
            self.score += 1;
            self.just_ate = true;
            self.food.spawn(&self.snake.body);
            return UpdateResult::Ate;
        }

        UpdateResult::Moved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    None,
    Moved,
    Ate,
    Died,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn grows_when_eating() {
        let mut game = Game::new(GameMode::Classic);
        let len_before = game.snake.body.len();
        let next = Snake::resolve_head(game.snake.next_head_coords(), GameMode::Classic).unwrap();
        game.food.position = next;
        game.update();
        assert_eq!(game.snake.body.len(), len_before + 1);
        assert_eq!(game.score, 1);
    }

    #[test]
    fn classic_wall_collision_ends_game() {
        let mut game = Game::new(GameMode::Classic);
        game.snake.body = vec![(0, 0)];
        game.snake.direction = Direction::Left;
        game.update();
        assert!(game.game_over);
    }

    #[test]
    fn wrap_mode_wraps_horizontally() {
        let mut snake = Snake::new(0, 5);
        snake.body = vec![(0, 5)];
        snake.direction = Direction::Left;
        let wrapped = Snake::resolve_head(snake.next_head_coords(), GameMode::Wrap).unwrap();
        assert_eq!(wrapped, (BOARD_WIDTH - 1, 5));
    }

    #[test]
    fn speed_increases_every_five_food() {
        let mut game = Game::new(GameMode::Classic);
        assert_eq!(game.tick_rate_ms(), BASE_TICK_RATE_MS);
        game.score = 5;
        assert!(game.tick_rate_ms() < BASE_TICK_RATE_MS);
        game.score = 100;
        assert_eq!(game.tick_rate_ms(), MIN_TICK_RATE_MS);
    }
}
