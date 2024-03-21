use rand::prelude::*;
use std::f32::consts::PI;
use std::time;

pub const SCREEN_WIDTH: i32 = 600;
pub const SCREEN_HEIGHT: i32 = 400;
pub const FPS: i32 = 60;
pub const GROUND_LENGTH: usize = 256;
pub const PLAYER_WIDTH: f32 = 30.0;
pub const PLAYER_HEIGHT: f32 = 30.0;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Command {
    pub left: i8,
    pub right: i8,
    pub up: i8,
    pub down: i8,
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
    pub is_playing: bool,
    pub score: i32,
    pub requested_sounds: Vec<&'static str>,
    pub requested_musics: Vec<&'static str>,
    pub player: Player,
    pub ground: [u8; GROUND_LENGTH],
    pub t: f32,     // 画面左端のワールド座標
    pub speed: f32, // スクロールスピード
    pub params: Params,
    pub music_started: bool,
}

pub struct Params {
    pub speed_scale: f32,
    pub gravity: f32,
    pub control_rotate_scale: f32,
    pub min_speed: f32,
}

impl Params {
    pub fn new() -> Self {
        Self {
            speed_scale: 7.0,
            gravity: 0.1,
            control_rotate_scale: 0.05,
            min_speed: 0.5,
        }
    }
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
            is_playing: true,
            score: 0,
            requested_sounds: Vec::new(),
            requested_musics: Vec::new(),

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
            params: Params::new(),
            music_started: false,
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

        if command.up == 1 && !self.music_started {
            self.music_started = true;
            self.requested_musics.push("play");
        }

        self.scroll(command);
        self.move_player(command);
        self.score = (self.t / 30.0) as i32;
    }

    pub fn scroll(&mut self, command: Command) {
        // speed = 0.9 * speed + 0.1 * (up - down) と計算するのと同じ。
        // speedの初期値が0なので、常に0〜1の範囲におさまる。
        // self.speed = 0.9 * self.speed + 0.1 * (command.up - command.down) as f32;
        let prev_speed = self.speed;
        self.speed -= (self.speed - (command.up - command.down) as f32) * 0.1;
        if self.speed < 0.0 {
            self.speed = 0.0;
        }
        // 摩擦による速度減少では、一定以下にならないようにする
        if command.down == 0
            && prev_speed >= self.params.min_speed
            && self.speed < self.params.min_speed
        {
            self.speed = self.params.min_speed;
        }
        self.t += self.params.speed_scale * self.speed;
    }

    pub fn move_player(&mut self, command: Command) {
        // プレイヤー位置の地面の高さ
        let p1 = self.ground_y(self.player.x);
        // プレイヤー位置よりちょっと先の地面の高さ
        let p2 = self.ground_y(self.player.x + 5.0);

        let grounded: bool;

        // プレイヤーの足元が地面より上なら
        if self.player.y + PLAYER_HEIGHT < p1 {
            grounded = false;
            self.player.y_speed += self.params.gravity;
        } else {
            grounded = true;
            self.player.y_speed -= self.player.y - (p1 - PLAYER_HEIGHT);
            self.player.y = p1 - PLAYER_HEIGHT;

            if self.player.rot.abs() > PI * 0.5 {
                if self.is_playing {
                    self.is_playing = false;
                    self.requested_musics.push("fadeout");
                }
            }
        }

        if self.is_playing {
            // p1とp2を結ぶ線分の角度（プレイヤーの角度）
            let angle = f32::atan2(p2 - PLAYER_HEIGHT - self.player.y, 5.0);

            self.player.y += self.player.y_speed;

            if grounded {
                self.player.rot -= (self.player.rot - angle) * 0.5;
                self.player.r_speed = self.player.r_speed - (angle - self.player.rot);
            }

            self.player.r_speed += (command.left - command.right) as f32 * 0.05;
            self.player.rot -= self.player.r_speed * self.params.control_rotate_scale;
        } else {
            // 転倒したら派手に回転させる
            self.player.x -= 5.0;
            self.player.r_speed = 0.15;
            self.player.rot -= self.player.r_speed;
            // 画面から消えたらゲームオーバー
            if self.player.x + PLAYER_WIDTH < 0.0 {
                self.is_over = true;
                self.requested_sounds.push("crash.wav");
            }
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

// コサインでなめらかにした線形補間
pub fn cos_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * (1.0 - f32::cos(t * PI)) / 2.0
}

pub fn rad2deg(rad: f32) -> f32 {
    rad / PI * 180.0
}
