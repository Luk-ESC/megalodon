use std::num::NonZeroUsize;

use double::{update_thread, Event};
use fastrand::Rng;
use gradient::{Gradient, Steps};
use grid::EMPTY;
use minifb::{Key, KeyRepeat, MouseButton, Window};
use radii::RadiusId;

mod double;
mod gradient;
mod grid;
mod radii;
mod resize;

static DEFAULT_ZOOM: usize = 3;

fn main() {
    let mut window = Window::new(
        "megalodon",
        800,
        600,
        minifb::WindowOptions {
            resize: true,
            ..Default::default()
        },
    )
    .unwrap();

    window.set_target_fps(120);

    let steps = Steps::new(300).unwrap();
    let mut rng = Rng::new();
    let mut gradient = Gradient::new(&mut rng, steps);

    let (sender, recv) = std::sync::mpsc::channel();
    std::thread::spawn(move || update_thread(recv));

    let mut zoom = DEFAULT_ZOOM;

    let mut mouse_position;
    let mut mouse_in_window;
    let mut mouse_clicked;
    let mut radius = RadiusId::default();
    let mut last_output_size = (800, 600);

    let mut pixel_buffer = vec![0u32; 800 * 600];
    let mut temporaries = vec![];

    while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            gradient = Gradient::new(&mut rng, steps);
        }

        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            pixel_buffer.fill(EMPTY);
            sender.send(Event::Clear).unwrap();
        }

        if window.is_key_pressed(Key::Up, KeyRepeat::No) {
            zoom += 1;
        }

        if window.is_key_pressed(Key::Down, KeyRepeat::No) && zoom > 1 {
            zoom -= 1;
        }

        if window.is_key_pressed(Key::W, KeyRepeat::No) {
            radius.next_bigger();
            sender.send(Event::Radius(radius)).unwrap();
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            radius.next_smaller();
            sender.send(Event::Radius(radius)).unwrap();
        }

        let new_pos = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
        mouse_position = (
            (new_pos.0 / zoom as f32) as i32,
            (new_pos.1 / zoom as f32) as i32,
        );
        mouse_in_window = window.get_mouse_pos(minifb::MouseMode::Discard).is_some();

        mouse_clicked = window.get_mouse_down(MouseButton::Left);

        let output_size = window.get_size();
        let output_size = ((output_size.0 / zoom) as u32, (output_size.1 / zoom) as u32);

        if mouse_clicked {
            let color = gradient.next_color();
            sender.send(Event::Spawn(color, mouse_position)).unwrap();
        }

        if output_size != last_output_size {
            sender
                .send(Event::Resize(
                    NonZeroUsize::new(output_size.0 as _).unwrap(),
                    NonZeroUsize::new(output_size.1 as _).unwrap(),
                ))
                .unwrap();

            resize::smart_resize(
                &mut pixel_buffer,
                (last_output_size.0 as usize, last_output_size.1 as usize),
                (output_size.0 as usize, output_size.1 as usize),
            );

            temporaries.clear();
        }
        last_output_size = output_size;

        double::render_to(
            &mut pixel_buffer,
            &mut temporaries,
            mouse_in_window,
            &gradient,
            mouse_position,
            NonZeroUsize::new(output_size.0 as usize).unwrap(),
            NonZeroUsize::new(output_size.1 as usize).unwrap(),
            radius.get(),
        );

        window
            .update_with_buffer(&pixel_buffer, output_size.0 as _, output_size.1 as _)
            .unwrap();
    }
}
