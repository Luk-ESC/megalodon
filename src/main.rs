use std::{
    num::{NonZeroU8, NonZeroUsize},
    time::{Duration, Instant},
};

use double::{update_thread, Event};
use gradient::Gradient;
use grid::Grid;
use rand::thread_rng;
use sdl2::{
    event::{Event as SdlEvent, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::{Color, PixelFormatEnum},
};

mod changelist;
mod double;
mod gradient;
mod grid;

static SIZES: &[(usize, usize)] = &[
    (25, 25),
    (50, 50),
    (100, 100),
    (200, 200),
    (400, 400),
    (800, 600),
    (1024, 768),
    (1280, 720),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
];

static RADII: &[f64] = &[1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0];

static DEFAULT_GRID: usize = 3;
static DEFAULT_RADIUS: usize = 2;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("megalodon", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut steps = NonZeroU8::new(200).unwrap();
    let mut gradient = Gradient::new(&mut thread_rng(), steps);

    let (sender, recv) = std::sync::mpsc::channel();
    std::thread::spawn(move || update_thread(recv));

    let texture_creator = canvas.texture_creator();

    let mut grid_size = DEFAULT_GRID;
    let mut texture = texture_creator
        .create_texture_streaming(None, SIZES[grid_size].0 as _, SIZES[grid_size].1 as _)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut mouse_position = (0, 0);
    let mut mouse_in_window = false;
    let mut mouse_clicked = false;
    let mut radius = DEFAULT_RADIUS;

    let mut i = 0;
    'running: loop {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit { .. }
                | SdlEvent::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                SdlEvent::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    gradient = Gradient::new(&mut thread_rng(), steps);
                }
                SdlEvent::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => sender.send(Event::Clear).unwrap(),
                SdlEvent::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    if grid_size < SIZES.len() - 1 {
                        grid_size += 1;

                        texture = texture_creator
                            .create_texture_streaming(
                                None,
                                SIZES[grid_size].0 as _,
                                SIZES[grid_size].1 as _,
                            )
                            .unwrap();

                        sender
                            .send(Event::Resize(
                                NonZeroUsize::new(SIZES[grid_size].0).unwrap(),
                                NonZeroUsize::new(SIZES[grid_size].1).unwrap(),
                            ))
                            .unwrap();
                    }
                }
                SdlEvent::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    if grid_size > 0 {
                        grid_size -= 1;
                        texture = texture_creator
                            .create_texture_streaming(
                                PixelFormatEnum::RGB888,
                                SIZES[grid_size].0 as _,
                                SIZES[grid_size].1 as _,
                            )
                            .unwrap();

                        sender
                            .send(Event::Resize(
                                NonZeroUsize::new(SIZES[grid_size].0).unwrap(),
                                NonZeroUsize::new(SIZES[grid_size].1).unwrap(),
                            ))
                            .unwrap();
                    }
                }

                SdlEvent::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    if radius < RADII.len() - 1 {
                        radius += 1;
                        sender.send(Event::Radius(RADII[radius])).unwrap();
                    }
                }

                SdlEvent::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if radius > 0 {
                        radius -= 1;
                        sender.send(Event::Radius(RADII[radius])).unwrap();
                    }
                }

                SdlEvent::MouseMotion { x, y, .. } => {
                    mouse_position = (x, y);
                }
                SdlEvent::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    mouse_position = (x, y);
                    mouse_clicked = true;
                }

                SdlEvent::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    mouse_clicked = false;
                }
                SdlEvent::Window { win_event, .. } => match win_event {
                    WindowEvent::Hidden => println!("Hidden"),
                    WindowEvent::Exposed => println!("Exposed"),
                    WindowEvent::SizeChanged(x, y) => println!("Resized: {}x{}", x, y),
                    WindowEvent::Enter => mouse_in_window = true,
                    WindowEvent::Leave => {
                        mouse_in_window = false;
                        mouse_clicked = false;
                    }
                    WindowEvent::Close => panic!("AHH"),
                    _ => {}
                },
                _ => {}
            }
        }

        let output_size = canvas.output_size().unwrap();

        let grid_aspect_ratio = SIZES[grid_size].0 as f32 / SIZES[grid_size].1 as f32;
        let window_aspect_ratio = output_size.0 as f32 / output_size.1 as f32;

        let mut padding_sides = 0u32;
        let mut padding_top_bottom = 0u32;

        if grid_aspect_ratio > window_aspect_ratio {
            padding_top_bottom =
                (output_size.1 as f32 - output_size.0 as f32 / grid_aspect_ratio) as u32 / 2;
        } else {
            padding_sides =
                (output_size.0 as f32 - output_size.1 as f32 * grid_aspect_ratio) as u32 / 2;
        }

        let mouse_position_adjusted = (
            (mouse_position.0 - padding_sides as i32)
                .max(0)
                .min(output_size.0 as i32 - padding_sides as i32 * 2),
            (mouse_position.1 - padding_top_bottom as i32)
                .max(0)
                .min(output_size.1 as i32 - padding_top_bottom as i32 * 2),
        );

        let mouse_position_adjusted = (
            mouse_position_adjusted.0 as f32 * SIZES[grid_size].0 as f32
                / (output_size.0 as f32 - padding_sides as f32 * 2.0),
            mouse_position_adjusted.1 as f32 * SIZES[grid_size].1 as f32
                / (output_size.1 as f32 - padding_top_bottom as f32 * 2.0),
        );

        let mouse_position = (
            mouse_position_adjusted.0.round() as i32,
            mouse_position_adjusted.1.round() as i32,
        );

        double::render_to(&mut texture, mouse_in_window, &gradient, mouse_position);

        let destination = sdl2::rect::Rect::new(
            padding_sides as i32,
            padding_top_bottom as i32,
            output_size.0 - padding_sides * 2,
            output_size.1 - padding_top_bottom * 2,
        );

        canvas.clear();
        canvas.copy(&texture, None, destination).unwrap();

        if mouse_clicked {
            let color = gradient.next_color();
            sender.send(Event::Spawn(color, mouse_position)).unwrap();
        }

        canvas.present();

        //println!("Frame time: {:#?}", start.elapsed());

        if i % 120 == 0 {
            println!("FPS: {}", 1.0 / start.elapsed().as_secs_f64());
        }
        i += 1;
    }
}
