use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

pub const TITLE: &str = "rust-motorcycle";

struct Image<'a> {
    texture: Texture<'a>,
    #[allow(dead_code)]
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
    fonts: HashMap<String, sdl2::ttf::Font<'a, 'a>>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(TITLE, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    sdl_context.mouse().show_cursor(false);

    init_mixer();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas, &ttf_context);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    'running: loop {
        let started = SystemTime::now();

        let mut command = Command::default();

        let keyboard_state = event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
            command.left = 1;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            command.right = 1;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
            command.up = 1;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
            command.down = 1;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    if code == Keycode::Escape {
                        break 'running;
                    }
                    match code {
                        Keycode::Space => {
                            if game.is_over {
                                game = Game::new();
                            }
                        }
                        _ => {}
                    };
                }
                _ => {}
            }
        }
        game.update(command);
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / model::FPS as u32);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    #[allow(unused_variables)] canvas: &mut Canvas<Window>,
    ttf_context: &'a Sdl2TtfContext,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
        fonts: HashMap::new(),
    };

    let entries = fs::read_dir("resources/image").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bmp") {
            let temp_surface = sdl2::surface::Surface::load_bmp(&path).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&temp_surface)
                .expect(&format!("cannot load image: {}", path_str));

            let basename = path.file_name().unwrap().to_str().unwrap();
            let image = Image::new(texture);
            resources.images.insert(basename.to_string(), image);
        }
    }

    let entries = fs::read_dir("./resources/sound").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".wav") {
            let chunk = mixer::Chunk::from_file(path_str)
                .expect(&format!("cannot load sound: {}", path_str));
            let basename = path.file_name().unwrap().to_str().unwrap();
            resources.chunks.insert(basename.to_string(), chunk);
        }
    }

    load_font(
        &mut resources,
        &ttf_context,
        "./resources/font/boxfont2.ttf",
        18,
        "boxfont",
    );

    resources
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    let clear_color = Color::RGB(0x11, 0x99, 0xff);
    let font = resources.fonts.get_mut("boxfont").unwrap();

    canvas.set_draw_color(clear_color);
    canvas.clear();

    // render player
    let player_image = resources.images.get_mut("moto.bmp").unwrap();
    canvas
        .copy_ex(
            &player_image.texture,
            None,
            Rect::new(
                game.player.x as i32,
                game.player.y as i32,
                PLAYER_WIDTH as u32,
                PLAYER_HEIGHT as u32,
            ),
            rad2deg(game.player.rot) as f64, /* SDLのangleは時計回りが正で度 */
            Point::new(PLAYER_WIDTH as i32 / 2, PLAYER_HEIGHT as i32 / 2),
            false,
            false,
        )
        .unwrap();

    // render ground
    canvas.set_draw_color(Color::BLACK);
    for i in 0..SCREEN_WIDTH {
        canvas
            .draw_line(
                Point::new(i as i32, SCREEN_HEIGHT),
                Point::new(i as i32, game.ground_y(i as f32) as i32),
            )
            .unwrap();
    }

    if game.is_over {
        canvas.set_draw_color(Color::RGBA(255, 0, 0, 128));
        canvas.fill_rect(Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))?;
    }

    render_font(
        canvas,
        &font,
        format!("{:10}", game.score).to_string(),
        SCREEN_WIDTH,
        0,
        Color::WHITE,
        TextPivot::Right,
    );

    canvas.present();

    Ok(())
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}

enum TextPivot {
    Left,   // 左寄せ。xは左端を指定
    Center, // 中央寄せ。xは中心を指定
    Right,  // 右寄せ。xは右端を指定
}

fn render_font(
    canvas: &mut Canvas<Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
    color: Color,
    align: TextPivot,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font.render(&text).blended(color).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let x: i32 = match align {
        TextPivot::Left => x,
        TextPivot::Center => x - texture.query().width as i32 / 2,
        TextPivot::Right => x - texture.query().width as i32,
    };
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, texture.query().width, texture.query().height),
        )
        .unwrap();
}

fn load_font<'a>(
    resources: &mut Resources<'a>,
    ttf_context: &'a Sdl2TtfContext,
    path_str: &str,
    point_size: u16,
    key: &str,
) {
    let font = ttf_context
        .load_font(path_str, point_size)
        .expect(&format!("cannot load font: {}", path_str));
    resources.fonts.insert(key.to_string(), font);
}
