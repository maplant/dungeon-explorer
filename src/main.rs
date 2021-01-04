mod kd_tree;
mod map_gen;
mod rect;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dungeon-explorer", about = "A nice random cavern screensaver")]
struct Opt {
    /// Activate fullscreen mode.
    #[structopt(short, long)]
    fullscreen: bool,

    /// Turn on dark mode. Defaults to off.
    #[structopt(short, long)]
    dark_mode: bool,

    /// Clears the screen and starts again when then screen is full.
    #[structopt(short, long)]
    restart: bool,

    /// Width of the screen. Defaults to 1024 if not fullscreen.
    #[structopt(short, long, required_if("height", "Some"))]
    width: Option<u32>,

    /// Height of the screen. Defaults to 728 if not fullscreen.
    #[structopt(short, long, required_if("width", "Some"))]
    height: Option<u32>,
}

fn main() {
    let opt = Opt::from_args();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let override_resolution = opt.width.is_some();
    let width = opt.width.unwrap_or(1024);
    let height = opt.height.unwrap_or(728);

    let mut window = video_subsystem.window("dungeon-explorer", width, height);
    let window = match (opt.fullscreen, override_resolution) {
        (true, false) => window.fullscreen_desktop(),
        (true, true) => window.fullscreen(),
        _ => &mut window,
    };
    let window = window.position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut map_gen = map_gen::MapGenerator::new(width, height, rand::thread_rng());

    let dirt_color = if opt.dark_mode {
        (0, 0, 0)
    } else {
        (u8::MAX, u8::MAX, u8::MAX)
    };

    canvas.set_draw_color(Color::RGB(dirt_color.0, dirt_color.1, dirt_color.2));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        i = (i + 1) % 255;
        if let Some(room) = map_gen.next() {
            room.draw(
                &mut canvas,
                (i as u8, rand::random(), (255 - i) as u8),
                dirt_color,
            )
        } else if opt.restart {
            canvas.set_draw_color(Color::RGB(dirt_color.0, dirt_color.1, dirt_color.2));
            drop(std::mem::replace(
                &mut map_gen,
                map_gen::MapGenerator::new(width, height, rand::thread_rng()),
            ));
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
