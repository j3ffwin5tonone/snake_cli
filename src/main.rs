mod app;
mod audio;
mod game;
mod input;
mod persist;
mod render;
mod session;

use macroquad::prelude::*;

use app::App;
use audio::Sounds;
use render::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    let sounds = Sounds::load().await;
    let mut app = App::new(sounds);

    loop {
        if !app.update() {
            break;
        }
        app.draw();
        next_frame().await;
    }
}
