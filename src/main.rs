use double::{update_thread, Event};
use fastrand::Rng;
use gradient::{Gradient, Steps};
use minifb::{Key, KeyRepeat, MouseButton, Window};
use radii::RadiusId;

mod double;
mod gradient;
mod grid;
mod radii;
mod resize;

static DEFAULT_WIDTH: usize = 800;
static DEFAULT_HEIGHT: usize = 600;

fn main() {
    let mut window = Window::new(
        "megalodon",
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
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

    let mut zoom = 3;
    let mut radius = RadiusId::default();
    let mut last_output_size = (DEFAULT_WIDTH as u16, DEFAULT_HEIGHT as u16);
    let mut pixel_buffer = vec![0u32; DEFAULT_WIDTH * DEFAULT_HEIGHT];
    let mut temporaries = vec![];

    while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            gradient = Gradient::new(&mut rng, steps);
        }

        if window.is_key_pressed(Key::C, KeyRepeat::No) {
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

        let mouse_position = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
        let mouse_position = (
            (mouse_position.0 / zoom as f32).round() as u16,
            (mouse_position.1 / zoom as f32).round() as u16,
        );

        if window.get_mouse_down(MouseButton::Left) {
            let color = gradient.next_color();
            sender.send(Event::Spawn(color, mouse_position)).unwrap();
        }

        let mouse_in_window = window.get_mouse_pos(minifb::MouseMode::Discard).is_some();

        let output_size = window.get_size();
        let output_size = ((output_size.0 / zoom) as u16, (output_size.1 / zoom) as u16);
        if output_size != last_output_size {
            sender
                .send(Event::Resize(output_size.0, output_size.1))
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
            output_size.0,
            output_size.1,
            radius.get(),
        );

        window
            .update_with_buffer(&pixel_buffer, output_size.0 as _, output_size.1 as _)
            .unwrap();
    }
}
