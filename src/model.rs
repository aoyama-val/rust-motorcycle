use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Left,
    Right,
    Up,
    Down,
}

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub rot: f32,
}

pub struct Game {
    pub rng: StdRng,
    pub frame: i32,
    pub is_over: bool,
    pub requested_sounds: Vec<&'static str>,
    pub player: Player,
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
                x: 0,
                y: 0,
                rot: 0.0,
            },
        };

        game
    }

    pub fn update(&mut self, command: Command) {
        self.frame += 1;

        if self.is_over {
            return;
        }

        match command {
            Command::None => {}
            Command::Left => todo!(),
            Command::Right => todo!(),
            Command::Up => todo!(),
            Command::Down => todo!(),
        }
    }
}
