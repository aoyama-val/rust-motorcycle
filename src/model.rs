use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const GROUND_LENGTH: usize = 256;

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
}

pub struct Game {
    pub rng: StdRng,
    pub frame: i32,
    pub is_over: bool,
    pub requested_sounds: Vec<&'static str>,
    pub player: Player,
    pub ground: [u8; GROUND_LENGTH],
    pub t: f32,
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
                x: 0.0,
                y: 0.0,
                rot: 0.0,
            },
            ground: [0; GROUND_LENGTH],
            t: 0.0,
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

        self.t += 1.0;

        match command {
            Command::None => {}
            Command::Left => todo!(),
            Command::Right => todo!(),
            Command::Up => todo!(),
            Command::Down => todo!(),
        }
    }

    pub fn noise(&self, x: f32) -> f32 {
        let x = x * 0.01 % 255.0;

        coslerp(
            self.ground[x.floor() as usize] as f32,
            self.ground[x.ceil() as usize] as f32,
            x - x.floor(),
        )
    }

    pub fn ground_y(&self, i: i32) -> f32 {
        self.noise(self.t + i as f32) * 0.25
    }
}

pub fn coslerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * (1.0 - f32::cos(t * std::f32::consts::PI)) / 2.0
}
