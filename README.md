# Snake

A windowed Snake game built with Rust and [macroquad](https://github.com/not-fl3/macroquad).

## Features

- Classic and Wrap game modes
- Progressive speed increase every 5 food eaten
- Smooth interpolated snake movement
- Sound effects for eating and game over
- Local top-5 leaderboard (persisted in `~/.snake_cli_leaderboard`)
- Start menu, countdown, pause, and game-over screens

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable)

## Build and run

```bash
cargo run
```

Release build:

```bash
cargo build --release
./target/release/snake_cli
```

## Controls

| Key | Action |
|-----|--------|
| WASD / Arrow keys | Move |
| M | Toggle Classic / Wrap mode (start menu) |
| Enter | Start / Play again |
| P | Pause / Resume |
| Q / Esc | Quit |

## Game modes

- **Classic** — hitting a wall ends the game
- **Wrap** — the snake wraps around to the opposite side

## Development

```bash
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

CI runs on every push to `main` via GitHub Actions.

## Version

Current version: **0.2.0**
