use rand::prelude::*;
use std::f32::consts::PI;
use std::time;

pub const SCREEN_WIDTH: i32 = 600;
pub const SCREEN_HEIGHT: i32 = 400;
pub const FPS: i32 = 60;
pub const GROUND_LENGTH: usize = 256;
pub const PLAYER_WIDTH: f32 = 30.0;
pub const PLAYER_HEIGHT: f32 = 30.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Left,
    Right,
    Up,
    Down,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub rot: f32,
    pub y_speed: f32,
    pub r_speed: f32,
}

pub struct Game {
    pub rng: StdRng,
    pub frame: i32,
    pub is_over: bool,
    pub requested_sounds: Vec<&'static str>,
    pub player: Player,
    pub ground: [u8; GROUND_LENGTH],
    pub t: f32,     // 画面左端のワールド座標
    pub speed: f32, // スクロールスピード
    pub speed_scale: f32,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);
        println!("random seed = {}", timestamp);
        // let rng = StdRng::seed_from_u64(0);

        let mut game = Game {
            rng: rng,
            frame: -1,
            is_over: false,
            requested_sounds: Vec::new(),
            player: Player {
                x: SCREEN_WIDTH as f32 / 2.0 - PLAYER_WIDTH / 2.0,
                y: 0.0,
                rot: 0.0,
                y_speed: 0.0,
                r_speed: 0.0,
            },
            ground: [0; GROUND_LENGTH],
            t: 0.0,
            speed: 0.0,
            speed_scale: 7.0,
        };

        game.create_stage();

        game
    }

    pub fn create_stage(&mut self) {
        for i in 0..self.ground.len() {
            self.ground[i] = i as u8;
        }
        self.ground.shuffle(&mut self.rng);
    }

    pub fn update(&mut self, command: Command) {
        self.frame += 1;

        if self.is_over {
            return;
        }

        let mut left_pressed = 0;
        let mut right_pressed = 0;
        let mut up_pressed = 0;
        let mut down_pressed = 0;

        match command {
            Command::None => {}
            Command::Left => left_pressed = 1,
            Command::Right => right_pressed = 1,
            Command::Up => up_pressed = 1,
            Command::Down => down_pressed = 1,
        }

        self.speed -= (self.speed - (up_pressed - down_pressed) as f32) * 0.1;
        if self.speed < 0.0 {
            self.speed = 0.0;
        }
        self.t += self.speed_scale * self.speed;

        let p1 = self.ground_y(self.player.x);
        let p2 = self.ground_y(self.player.x + 5.0);

        let grounded: bool;

        // プレイヤーの足元が地面より上なら
        if self.player.y + PLAYER_HEIGHT < p1 {
            grounded = false;
            self.player.y_speed += 0.1;
        } else {
            grounded = true;
            self.player.y_speed -= self.player.y - (p1 - PLAYER_HEIGHT);
            self.player.y = p1 - PLAYER_HEIGHT;

            if self.player.rot.abs() > PI * 0.5 {
                self.is_over = true;
                self.requested_sounds.push("crash.wav");
                return;
            }
        }

        let angle = f32::atan2(p2 - PLAYER_HEIGHT - self.player.y, 5.0);

        self.player.y += self.player.y_speed;

        if grounded {
            self.player.rot -= (self.player.rot - angle) * 0.5;
            self.player.r_speed = self.player.r_speed - (angle - self.player.rot);
        }

        self.player.r_speed += (left_pressed - right_pressed) as f32 * 0.05;
        self.player.rot -= self.player.r_speed * 0.1;

        if self.player.rot > PI {
            self.player.rot = -PI;
        }
        if self.player.rot < -PI {
            self.player.rot = PI;
        }
    }

    pub fn noise(&self, x: f32) -> f32 {
        let x = x * 0.01 % 255.0;

        cos_lerp(
            self.ground[x.floor() as usize] as f32,
            self.ground[x.ceil() as usize] as f32,
            x - x.floor(),
        )
    }

    pub fn ground_y(&self, x: f32) -> f32 {
        SCREEN_HEIGHT as f32 - self.noise(self.t + x) * 0.25
    }
}

pub fn cos_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * (1.0 - f32::cos(t * PI)) / 2.0
}

pub fn rad2deg(rad: f32) -> f32 {
    rad / PI * 180.0
}
