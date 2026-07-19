# Snake v0.2.0 — CRT Phosphor Terminal

A windowed Snake game built with Rust and [macroquad](https://github.com/not-fl3/macroquad). Dark green/amber palette, glow text, scanlines, and terminal-style menus.

## Downloads

| Platform | File |
|----------|------|
| **Windows x86_64** | [`snake_cli-0.2.0-windows-x86_64.zip`](https://github.com/j3ffwin5tonone/snake_cli/releases/download/v0.2.0/snake_cli-0.2.0-windows-x86_64.zip) |
| **Windows x86_64 (exe)** | [`snake_cli-0.2.0-windows-x86_64.exe`](https://github.com/j3ffwin5tonone/snake_cli/releases/download/v0.2.0/snake_cli-0.2.0-windows-x86_64.exe) |

Unzip (or download the `.exe` directly), then run `snake_cli.exe`. No install required.

## Screenshots

| Start menu | Gameplay / HUD |
|------------|----------------|
| ![Start menu](https://raw.githubusercontent.com/j3ffwin5tonone/snake_cli/main/assets/start-menu.png) | ![Gameplay HUD](https://raw.githubusercontent.com/j3ffwin5tonone/snake_cli/main/assets/playing-hud.png) |

## Features

- **Classic**, **Wrap**, and **Daily Challenge** game modes
- **Board presets**: Classic (20×10), Compact (15×8), Arena (25×15)
- **Special food**: Normal (+1), Golden (+5), Speed Boost (+2 with temporary faster ticks)
- **Combo streaks**: eat quickly for bonus points and streak multipliers
- Progressive speed increase every 5 food eaten (Classic/Wrap)
- Input buffer (1–2 turns queued) for fair high-speed play
- Win condition when the board is completely filled
- Smooth interpolated snake movement
- Sound effects for eating, level-up, and game over
- Per-mode top-5 leaderboards (persisted in `~/.snake_cli_leaderboard`)
- 8 achievements (persisted in `~/.snake_cli_achievements`)
- Run statistics on game over (food eaten, max streak, time, vs. personal best)
- Start menu, countdown, pause, level-up overlay, and victory/game-over screens

## Controls

| Key | Action |
|-----|--------|
| WASD / Arrow keys | Move |
| Tab | Edit / finish editing name |
| M | Toggle Classic / Wrap / Daily mode (start menu) |
| B | Toggle board preset (start menu) |
| Enter | Start / Play again |
| P | Pause / Resume |
| T | Toggle sound |
| R | Play again (game over) |
| Q / Esc | Quit |

## Game modes

- **Classic** — hitting a wall ends the game
- **Wrap** — the snake wraps around to the opposite side
- **Daily** — fixed tick rate, same food sequence for everyone each day (seeded RNG)

## Build from source

Requires [Rust](https://www.rust-lang.org/tools/install) (stable):

```bash
cargo build --release
./target/release/snake_cli
```
