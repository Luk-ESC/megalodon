use std::num::NonZeroUsize;

use fastrand::Rng;

use crate::{
    changelist::{Change, Changelist, Direction},
    gradient::Gradient,
    resize,
};

fn circle_offsets(radius: f64) -> impl Iterator<Item = (isize, isize)> {
    let radius_ceil = radius.ceil() as isize;
    let sq_radius = radius * radius;

    (-radius_ceil + 1..radius_ceil)
        .flat_map(move |y| (-radius_ceil + 1..radius_ceil).map(move |x| (x, y)))
        .filter(move |(x, y)| {
            let x = *x as f64;
            let y = *y as f64;
            x * x + y * y <= sq_radius
        })
}

#[derive(Clone)]
pub struct Grid {
    width: NonZeroUsize,
    height: NonZeroUsize,
    radius: f64,
    colors: Vec<u32>,
    rng: Rng,
}

pub const EMPTY: u32 = 0xFFE0FFFE;

impl Grid {
    pub fn new(width: NonZeroUsize, height: NonZeroUsize, radius: f64) -> Self {
        assert!(radius > 0.0);
        Self {
            width,
            height,
            radius,
            colors: vec![EMPTY; width.get() * height.get()],
            rng: Rng::new(),
        }
    }

    pub fn update(&mut self, changes: &mut Changelist) -> bool {
        let mut row = self.height.get() - 2;

        let mut updated = false;

        loop {
            let direction = if self.rng.bool() { 1 } else { -1 };

            let mut column = if direction > 0 {
                0
            } else {
                self.width.get() as i64 - 1
            };

            let limit = if direction > 0 {
                self.width.get() - 1
            } else {
                0
            };

            loop {
                let i = row * self.width.get() + column as usize;

                if !self.is_empty(i) {
                    let moved_down = self.update_pixel(i, changes);
                    updated |= moved_down;
                }

                if column as usize == limit {
                    break;
                }

                column += direction;
            }

            if row == 0 {
                break;
            }

            row -= 1;
        }

        updated
    }

    fn update_pixel(&mut self, i: usize, changes: &mut Changelist) -> bool {
        let below = i + self.width.get();
        let below_left = below - 1;
        let below_right = below + 1;

        let below_row = below / self.width.get();

        // If there are no pixels below, move it down.
        if self.is_empty(below) {
            self.move_(i, below, changes);
            return true;
        } else if below_left / self.width.get() == below_row && self.is_empty(below_left) {
            self.move_(i, below_left, changes);
            return true;
        } else if below_right / self.width.get() == below_row && self.is_empty(below_right) {
            self.move_(i, below_right, changes);
            return true;
        }
        false
    }

    fn raw_clear(&mut self) {
        self.colors.fill(EMPTY);
    }

    pub fn clear(&mut self, changes: &mut Changelist) {
        self.raw_clear();
        changes.clear();
    }

    fn raw_move(&mut self, a: usize, b: usize) {
        assert!(!self.is_empty(a));

        self.colors[b] = self.colors[a];
        self.colors[a] = EMPTY;
    }

    pub fn move_(&mut self, a: usize, b: usize, changes: &mut Changelist) {
        self.raw_move(a, b);

        if a + self.width.get() == b {
            changes.move_pixel(a, Direction::Down);
        } else if a + self.width.get() + 1 == b {
            changes.move_pixel(a, Direction::DownRight);
        } else if a + self.width.get() - 1 == b {
            changes.move_pixel(a, Direction::DownLeft);
        } else {
            unreachable!("invalid move: {a} to {b}")
        }
    }

    fn raw_set_pixel(&mut self, a: usize, color: u32) {
        assert!(self.is_empty(a));

        self.colors[a] = color;
    }

    pub fn set_pixel(&mut self, a: usize, color: u32, changes: &mut Changelist) {
        self.raw_set_pixel(a, color);
        changes.spawn(a, color);
    }

    pub fn is_empty(&self, a: usize) -> bool {
        self.colors[a] == EMPTY
    }

    pub fn render_to(
        &self,
        buffer: &mut [u32],
        mouse_in_window: bool,
        gradient: &Gradient,
        mouse_position: (i32, i32),
    ) {
        if buffer.len() != self.colors.len() {
            return;
        }

        buffer.copy_from_slice(&self.colors);

        if mouse_in_window {
            let color = gradient.peek_color();
            for (dx, dy) in circle_offsets(self.radius) {
                let x = mouse_position.0 as isize + dx;
                let y = mouse_position.1 as isize + dy;

                if x >= 0
                    && x < self.width.get() as isize
                    && y >= 0
                    && y < self.height.get() as isize
                {
                    let i = (y * self.width.get() as isize + x) as usize;
                    if self.is_empty(i) {
                        buffer[i] = color;
                    }
                }
            }
        }
    }

    pub(crate) fn spawn(
        &mut self,
        mouse_pos: (i32, i32),
        color: u32,
        changes: &mut Changelist,
    ) -> bool {
        let mut placed_pixels = false;
        for (dx, dy) in circle_offsets(self.radius) {
            let x = mouse_pos.0 as isize + dx;
            let y = mouse_pos.1 as isize + dy;

            if x >= 0 && x < self.width.get() as isize && y >= 0 && y < self.height.get() as isize {
                let index = (y * self.width.get() as isize + x) as usize;
                if self.is_empty(index) {
                    self.set_pixel(index, color, changes);
                    placed_pixels = true;
                }
            }
        }
        placed_pixels
    }

    fn raw_resize(&mut self, width: NonZeroUsize, height: NonZeroUsize) {
        //let new_colors = vec![EMPTY; width.get() * height.get()].into_boxed_slice();
        //self.colors = new_colors;
        resize::smart_resize(
            &mut self.colors,
            (self.width.into(), self.height.into()),
            (width.into(), height.into()),
        );
        self.width = width;
        self.height = height;
    }

    pub fn resize(&mut self, width: NonZeroUsize, height: NonZeroUsize, changes: &mut Changelist) {
        self.raw_resize(width, height);

        changes.resize(width.get(), height.get());
    }

    pub fn apply_changes(&mut self, changes: impl Iterator<Item = Change>) {
        for change in changes {
            match change {
                Change::Clear => self.raw_clear(),
                Change::Resize(width, height) => self.raw_resize(
                    NonZeroUsize::new(width).unwrap(),
                    NonZeroUsize::new(height).unwrap(),
                ),
                Change::Move(pos, direction) => {
                    let pos2 = match direction {
                        Direction::Down => pos + self.width.get(),
                        Direction::DownLeft => pos + self.width.get() - 1,
                        Direction::DownRight => pos + self.width.get() + 1,
                    };
                    self.raw_move(pos, pos2);
                }
                Change::Spawn { color, pos } => {
                    self.raw_set_pixel(pos, color);
                }
            }
        }
    }

    pub(crate) fn set_radius(&mut self, r: f64) {
        self.radius = r;
    }

    pub(crate) fn radius(&self) -> f64 {
        self.radius
    }
}
