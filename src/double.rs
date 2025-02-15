use std::{
    num::NonZeroUsize,
    sync::{mpsc::Receiver, Mutex},
    time::{Duration, Instant},
};

use crate::{
    gradient::Gradient,
    grid::{circle_offsets, Grid, EMPTY},
    DEFAULT_RADIUS, RADII,
};

pub enum Event {
    Clear,
    Resize(NonZeroUsize, NonZeroUsize),
    Spawn(u32, (i32, i32)),
    Radius(f64),
}

static PIXELS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

pub fn update_thread(recv: Receiver<Event>) {
    let sleep_time = Duration::from_secs(1) / 120;
    let mut grid = Grid::new(
        NonZeroUsize::new(800).unwrap(),
        NonZeroUsize::new(600).unwrap(),
        RADII[DEFAULT_RADIUS],
    );
    {
        PIXELS.lock().unwrap().clone_from(&grid.colors);
    }

    let mut i = 0;
    let mut needs_update = false;
    loop {
        let start = Instant::now();

        let mut any_event = false;
        while let Ok(event) = recv.try_recv() {
            any_event = true;
            match event {
                Event::Clear => grid.clear(),
                Event::Resize(width, height) => {
                    needs_update = true;
                    grid.resize(width, height);
                }
                Event::Spawn(color, pos) => {
                    needs_update |= grid.spawn(pos, color);
                }
                Event::Radius(r) => {
                    grid.set_radius(r);
                }
            }
        }

        let mut updated = false;
        if needs_update {
            needs_update = grid.update();
            updated = true;
        }

        if any_event || updated {
            {
                PIXELS.lock().unwrap().clone_from(&grid.colors);
            }
        }

        let elapsed = start.elapsed();

        if elapsed < sleep_time {
            std::thread::sleep(sleep_time - elapsed);
        } else if i % 10 == 0 {
            println!("update took too long: {:?}", elapsed);
        }

        i += 1;
    }
}

pub fn render_to(
    buffer: &mut [u32],
    mouse_in_window: bool,
    gradient: &Gradient,
    mouse_position: (i32, i32),
    width: NonZeroUsize,
    height: NonZeroUsize,
    radius: f64,
) {
    {
        let colors = PIXELS.lock().unwrap();
        if buffer.len() != colors.len() {
            return;
        }

        buffer.copy_from_slice(&colors);
    }

    if mouse_in_window {
        let color = gradient.peek_color();
        for (dx, dy) in circle_offsets(radius) {
            let x = mouse_position.0 as isize + dx;
            let y = mouse_position.1 as isize + dy;

            if x >= 0 && x < width.get() as isize && y >= 0 && y < height.get() as isize {
                let i = (y * width.get() as isize + x) as usize;
                if buffer[i] == EMPTY {
                    buffer[i] = color;
                }
            }
        }
    }
}
