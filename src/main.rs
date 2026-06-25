use macroquad::prelude::*;
use ::rand::RngExt;
use ::rand::rng;
use std::fs;
use std::path::PathBuf;

// File name used to persist the all-time highscore.
const HIGHSCORE_FILE: &str = ".snake_cli_highscore";

// Resolve the highscore file path inside the user's home directory, falling
// back to the current directory if HOME is unavailable.
fn highscore_path() -> PathBuf {
    let mut path = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path.push(HIGHSCORE_FILE);
    path
}

fn load_highscore() -> u16 {
    fs::read_to_string(highscore_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_highscore(score: u16) {
    // Best-effort persistence; ignore errors so a write failure never crashes
    // the game.
    let _ = fs::write(highscore_path(), score.to_string());
}

// Board dimensions
const BOARD_WIDTH: u16 = 20;
const BOARD_HEIGHT: u16 = 10;

// Game tick rate in milliseconds
const TICK_RATE_MS: u64 = 150;

// Snake direction
#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

// Snake struct
#[derive(Debug, Clone)]
struct Snake {
    body: Vec<(u16, u16)>,
    direction: Direction,
}

impl Snake {
    fn new(start_x: u16, start_y: u16) -> Self {
        Snake {
            // saturating_sub guards against underflow if start_x == 0
            body: vec![(start_x, start_y), (start_x.saturating_sub(1), start_y)],
            direction: Direction::Right,
        }
    }

    fn head(&self) -> (u16, u16) {
        *self.body.first().unwrap()
    }

    fn next_head(&self) -> (u16, u16) {
        let (head_x, head_y) = self.head();
        match self.direction {
            Direction::Up => (head_x, head_y.saturating_sub(1)),
            Direction::Down => (head_x, head_y.saturating_add(1)),
            Direction::Left => (head_x.saturating_sub(1), head_y),
            Direction::Right => (head_x.saturating_add(1), head_y),
        }
    }

    // Move forward; if `grew` is true, keep the tail (snake grows by 1).
    fn advance(&mut self, grew: bool) {
        let new_head = self.next_head();
        self.body.insert(0, new_head);
        if !grew {
            self.body.pop();
        }
    }

    fn change_direction(&mut self, new_dir: Direction) {
        // Prevent reversing direction
        if new_dir != self.direction.opposite() {
            self.direction = new_dir;
        }
    }

    fn check_self_collision(&self) -> bool {
        let head = self.head();
        self.body.iter().skip(1).any(|&segment| segment == head)
    }
}

// Food struct
#[derive(Debug, Clone)]
struct Food {
    position: (u16, u16),
}

impl Food {
    fn new(snake_body: &[(u16, u16)]) -> Self {
        let mut food = Food { position: (0, 0) };
        food.spawn(snake_body);
        food
    }

    fn spawn(&mut self, snake_body: &[(u16, u16)]) {
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

// Game struct
#[derive(Debug)]
struct Game {
    snake: Snake,
    food: Food,
    score: u16,
    highscore: u16,
    game_over: bool,
}

impl Game {
    fn new() -> Self {
        let start_x = 5;
        let start_y = BOARD_HEIGHT / 2;
        let snake = Snake::new(start_x, start_y);
        let food = Food::new(&snake.body);

        Game {
            snake,
            food,
            score: 0,
            highscore: load_highscore(),
            game_over: false,
        }
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        // Determine whether the next head lands on food BEFORE moving,
        // so growth is handled in a single advance step (no tail jump).
        let next = self.snake.next_head();
        let will_eat = next == self.food.position;

        self.snake.advance(will_eat);

        if self.check_collision() {
            self.game_over = true;
            if self.score > self.highscore {
                self.highscore = self.score;
                save_highscore(self.highscore);
            }
            return;
        }

        if will_eat {
            self.score += 1;
            self.food.spawn(&self.snake.body);
        }
    }

    fn check_collision(&self) -> bool {
        let (head_x, head_y) = self.snake.head();

        // Wall collision (x/y are u16; saturating_sub means a left/up hit
        // produces 0, which is still inside the board, so we rely on the
        // movement having clamped — wall hit is when head reaches the edge
        // and the next step would exceed it).
        if head_x >= BOARD_WIDTH || head_y >= BOARD_HEIGHT {
            return true;
        }

        self.snake.check_self_collision()
    }
}

// Rendering layout (pixels)
const CELL_SIZE: f32 = 30.0;
// Vertical space reserved at the top for the score/highscore text.
const UI_STRIP: f32 = 50.0;
// Padding around the board.
const MARGIN: f32 = 20.0;

const WINDOW_WIDTH: i32 = (BOARD_WIDTH as f32 * CELL_SIZE + MARGIN * 2.0) as i32;
const WINDOW_HEIGHT: i32 = (BOARD_HEIGHT as f32 * CELL_SIZE + UI_STRIP + MARGIN * 2.0) as i32;

const TICK_SECONDS: f32 = TICK_RATE_MS as f32 / 1000.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

// Translate a board cell into its top-left pixel coordinates.
fn cell_to_pixel(x: u16, y: u16) -> (f32, f32) {
    let px = MARGIN + x as f32 * CELL_SIZE;
    let py = MARGIN + UI_STRIP + y as f32 * CELL_SIZE;
    (px, py)
}

// Apply steering input. Only the most recent of the pressed keys this frame
// takes effect; reversing into the snake is rejected by `change_direction`.
fn handle_steering(game: &mut Game) {
    if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
        game.snake.change_direction(Direction::Up);
    }
    if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
        game.snake.change_direction(Direction::Down);
    }
    if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
        game.snake.change_direction(Direction::Left);
    }
    if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
        game.snake.change_direction(Direction::Right);
    }
}

fn draw_game(game: &Game) {
    clear_background(Color::from_rgba(18, 18, 24, 255));

    // Score / highscore strip.
    let header = format!(
        "Score: {}    Highscore: {}",
        game.score,
        game.highscore.max(game.score)
    );
    draw_text(&header, MARGIN, MARGIN + 24.0, 30.0, WHITE);

    // Board background and border.
    let board_w = BOARD_WIDTH as f32 * CELL_SIZE;
    let board_h = BOARD_HEIGHT as f32 * CELL_SIZE;
    draw_rectangle(
        MARGIN,
        MARGIN + UI_STRIP,
        board_w,
        board_h,
        Color::from_rgba(30, 30, 40, 255),
    );
    draw_rectangle_lines(
        MARGIN,
        MARGIN + UI_STRIP,
        board_w,
        board_h,
        2.0,
        Color::from_rgba(80, 80, 100, 255),
    );

    // Food.
    let (fx, fy) = cell_to_pixel(game.food.position.0, game.food.position.1);
    draw_rectangle(
        fx + 3.0,
        fy + 3.0,
        CELL_SIZE - 6.0,
        CELL_SIZE - 6.0,
        Color::from_rgba(230, 70, 70, 255),
    );

    // Snake body (head drawn in a brighter shade).
    let head = game.snake.head();
    for &(x, y) in &game.snake.body {
        let (px, py) = cell_to_pixel(x, y);
        let color = if (x, y) == head {
            Color::from_rgba(120, 230, 120, 255)
        } else {
            Color::from_rgba(70, 180, 70, 255)
        };
        draw_rectangle(px + 1.0, py + 1.0, CELL_SIZE - 2.0, CELL_SIZE - 2.0, color);
    }

    if game.game_over {
        draw_game_over_overlay(game);
    }
}

fn draw_game_over_overlay(game: &Game) {
    // Dim the whole window to focus attention on the menu.
    draw_rectangle(
        0.0,
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        Color::from_rgba(0, 0, 0, 160),
    );

    let center_x = WINDOW_WIDTH as f32 / 2.0;

    // Simple horizontally-centered text helper.
    let centered = |text: &str, y: f32, size: f32, color: Color| {
        let dims = measure_text(text, None, size as u16, 1.0);
        draw_text(text, center_x - dims.width / 2.0, y, size, color);
    };

    centered("GAME OVER", WINDOW_HEIGHT as f32 / 2.0 - 60.0, 48.0, WHITE);

    let final_line = format!("Final Score: {}", game.score);
    centered(&final_line, WINDOW_HEIGHT as f32 / 2.0 - 20.0, 28.0, WHITE);

    if game.score >= game.highscore && game.score > 0 {
        centered(
            "New Highscore!",
            WINDOW_HEIGHT as f32 / 2.0 + 12.0,
            28.0,
            Color::from_rgba(255, 215, 0, 255),
        );
    } else {
        let hs = format!("Highscore: {}", game.highscore);
        centered(&hs, WINDOW_HEIGHT as f32 / 2.0 + 12.0, 28.0, WHITE);
    }

    centered(
        "Play again? [Enter] Yes    [Q] Quit",
        WINDOW_HEIGHT as f32 / 2.0 + 56.0,
        24.0,
        Color::from_rgba(200, 200, 200, 255),
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    // Accumulates frame time; the game advances one step per TICK_SECONDS.
    let mut elapsed: f32 = 0.0;

    loop {
        // Quit from anywhere.
        if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape) {
            break;
        }

        if game.game_over {
            // Game-over menu: restart or quit.
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::R) {
                game = Game::new();
                elapsed = 0.0;
            }
        } else {
            handle_steering(&mut game);

            elapsed += get_frame_time();
            while elapsed >= TICK_SECONDS {
                elapsed -= TICK_SECONDS;
                game.update();
                if game.game_over {
                    break;
                }
            }
        }

        draw_game(&game);
        next_frame().await;
    }
}