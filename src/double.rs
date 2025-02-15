use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Mutex,
    },
    time::{Duration, Instant},
};

use crate::{
    gradient::Gradient,
    grid::{circle_offsets, Grid, EMPTY},
    radii::RadiusId,
};

pub enum Event {
    Clear,
    Resize(u16, u16),
    Spawn(u32, (u16, u16)),
    Radius(RadiusId),
}

static CHANGED: AtomicBool = AtomicBool::new(false);
static PIXELS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

pub fn update_thread(recv: Receiver<Event>) {
    let sleep_time = Duration::from_secs(1) / 120;
    let mut grid = Grid::new();
    {
        PIXELS.lock().unwrap().clone_from(&grid.colors);
        CHANGED.store(true, Ordering::Relaxed);
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
                    grid.set_radius(r.get());
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
            CHANGED.store(true, Ordering::Relaxed);
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

#[expect(clippy::too_many_arguments)]
pub fn render_to(
    buffer: &mut Vec<u32>,
    temporaries: &mut Vec<usize>,
    mouse_in_window: bool,
    gradient: &Gradient,
    mouse_position: (u16, u16),
    width: u16,
    height: u16,
    radius: f64,
) {
    for i in temporaries.drain(..) {
        buffer[i] = EMPTY;
    }

    if CHANGED.swap(false, Ordering::Relaxed) {
        let mut lock = PIXELS.lock().unwrap();
        if lock.len() == buffer.len() {
            std::mem::swap(buffer, &mut *lock);
        }
        drop(lock);
    }

    if mouse_in_window {
        let color = gradient.peek_color();
        for (dx, dy) in circle_offsets(radius) {
            let x = mouse_position.0 as isize + dx;
            let y = mouse_position.1 as isize + dy;

            if x >= 0 && x < width as isize && y >= 0 && y < height as isize {
                let i = (y * width as isize + x) as usize;
                if buffer[i] == EMPTY {
                    temporaries.push(i);
                    buffer[i] = color;
                }
            }
        }
    }
}
