use macroquad::prelude::*;

pub struct VisualState {
    pub prev_body: Vec<(u16, u16)>,
    pub eat_flash: f32,
    pub death_shake: f32,
    pub level_up_flash: f32,
}

impl Default for VisualState {
    fn default() -> Self {
        VisualState {
            prev_body: Vec::new(),
            eat_flash: 0.0,
            death_shake: 0.0,
            level_up_flash: 0.0,
        }
    }
}

impl VisualState {
    pub fn snapshot_body(&mut self, body: &[(u16, u16)]) {
        self.prev_body = body.to_vec();
    }

    pub fn trigger_eat(&mut self) {
        self.eat_flash = 1.0;
    }

    pub fn trigger_death(&mut self) {
        self.death_shake = 1.0;
    }

    pub fn trigger_level_up(&mut self) {
        self.level_up_flash = 1.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.eat_flash = (self.eat_flash - dt * 3.0).max(0.0);
        self.death_shake = (self.death_shake - dt * 2.5).max(0.0);
        self.level_up_flash = (self.level_up_flash - dt * 2.0).max(0.0);
    }

    pub fn shake_offset(&self) -> f32 {
        if self.death_shake > 0.0 {
            (get_time() as f32 * 40.0).sin() * self.death_shake * 6.0
        } else {
            0.0
        }
    }
}

pub fn draw_level_up(timer: f32) {
    let alpha = (timer / 0.8).min(1.0);
    let overlay = Color::new(
        super::CRT_AMBER.r,
        super::CRT_AMBER.g,
        super::CRT_AMBER.b,
        alpha * 0.12,
    );
    draw_rectangle(
        0.0,
        0.0,
        super::WINDOW_WIDTH as f32,
        super::WINDOW_HEIGHT as f32,
        overlay,
    );
    super::centered_glow_text(
        "LEVEL UP",
        super::WINDOW_HEIGHT as f32 / 2.0 - 20.0,
        44.0,
        super::CRT_AMBER,
    );
}

pub fn draw_eat_flash(board_x: f32, board_y: f32, board_w: f32, board_h: f32, flash: f32) {
    if flash > 0.0 {
        draw_rectangle(
            board_x,
            board_y,
            board_w,
            board_h,
            Color::new(
                super::CRT_GREEN.r,
                super::CRT_GREEN.g,
                super::CRT_GREEN.b,
                flash * 0.12,
            ),
        );
    }
}

pub fn draw_level_up_edge(board_x: f32, board_y: f32, board_w: f32, board_h: f32, flash: f32) {
    if flash > 0.0 {
        let c = Color::new(
            super::CRT_AMBER.r,
            super::CRT_AMBER.g,
            super::CRT_AMBER.b,
            flash * 0.8,
        );
        draw_rectangle_lines(board_x, board_y, board_w, board_h, 3.0, c);
    }
}
