use std::{
    num::NonZeroUsize,
    sync::{atomic::AtomicPtr, mpsc::Receiver, LazyLock, Mutex},
    time::{Duration, Instant},
};

use sdl2::render::Texture;

use crate::{
    changelist::{self, Changelist},
    gradient::Gradient,
    grid::Grid,
    DEFAULT_GRID, DEFAULT_RADIUS, RADII, SIZES,
};

static WRITEBACK_GRID: LazyLock<Mutex<Grid>> = LazyLock::new(|| {
    Mutex::new(Grid::new(
        NonZeroUsize::new(SIZES[DEFAULT_GRID].0).unwrap(),
        NonZeroUsize::new(SIZES[DEFAULT_GRID].1).unwrap(),
        RADII[DEFAULT_RADIUS],
    ))
});

pub enum Event {
    Clear,
    Resize(NonZeroUsize, NonZeroUsize),
    Spawn(u32, (i32, i32)),
    Radius(f64),
}

pub fn update_thread(recv: Receiver<Event>) {
    let sleep_time = Duration::from_secs(1) / 120;
    let mut grid = WRITEBACK_GRID.lock().unwrap().clone();
    let mut changes = Changelist::new();

    let mut i = 0;
    let mut needs_update = false;
    loop {
        let start = Instant::now();

        let mut any_event = false;
        while let Ok(event) = recv.try_recv() {
            any_event = true;
            match event {
                Event::Clear => grid.clear(&mut changes),
                Event::Resize(width, height) => {
                    // TODO: ideally not
                    needs_update = true;
                    grid.resize(width, height, &mut changes);
                }
                Event::Spawn(color, pos) => {
                    needs_update |= grid.spawn(pos, color, &mut changes);
                }
                Event::Radius(r) => {
                    grid.set_radius(r);
                }
            }
        }

        let mut updated = false;
        if needs_update {
            needs_update = grid.update(&mut changes);
            updated = true;
        }

        if any_event || updated {
            {
                let mut lock = WRITEBACK_GRID.lock().unwrap();
                lock.apply_changes(changes.iter());
                lock.set_radius(grid.radius())
            }
            changes = Changelist::using_capacity(changes);
        }

        let elapsed = start.elapsed();

        if elapsed < sleep_time {
            std::thread::sleep(sleep_time - elapsed);
            if i % 20 == 0 && (updated) {
                println!("update took {elapsed:?}");
            }
        } else if i % 10 == 0 {
            println!("update took too long: {:?}", elapsed);
        }

        i += 1;
    }
}

pub fn render_to(
    texture: &mut Texture,
    mouse_in_window: bool,
    gradient: &Gradient,
    mouse_position: (i32, i32),
) {
    WRITEBACK_GRID
        .lock()
        .unwrap()
        .render_to(texture, mouse_in_window, gradient, mouse_position)
}
