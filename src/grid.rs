use std::num::NonZeroUsize;

use fastrand::Rng;

use crate::resize;

pub fn circle_offsets(radius: f64) -> impl Iterator<Item = (isize, isize)> {
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
    pub colors: Vec<u32>,
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

    pub fn update(&mut self) -> bool {
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
                    let moved_down = self.update_pixel(i, column);
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

    fn update_pixel(&mut self, i: usize, column: i64) -> bool {
        let below = i + self.width.get();
        let below_left = below - 1;
        let below_right = below + 1;

        // If there are no pixels below, move it down.
        if self.is_empty(below) {
            self.move_(i, below);
            return true;
        } else if column != 0 && self.is_empty(below_left) {
            self.move_(i, below_left);
            return true;
        } else if column != (self.width.get() - 1) as i64 && self.is_empty(below_right) {
            self.move_(i, below_right);
            return true;
        }
        false
    }

    pub fn clear(&mut self) {
        self.colors.fill(EMPTY);
    }

    fn move_(&mut self, a: usize, b: usize) {
        self.colors[b] = self.colors[a];
        self.colors[a] = EMPTY;
    }

    pub fn set_pixel(&mut self, a: usize, color: u32) {
        assert!(self.is_empty(a));

        self.colors[a] = color;
    }

    pub fn is_empty(&self, a: usize) -> bool {
        self.colors[a] == EMPTY
    }

    pub fn spawn(&mut self, mouse_pos: (i32, i32), color: u32) -> bool {
        let mut placed_pixels = false;
        for (dx, dy) in circle_offsets(self.radius) {
            let x = mouse_pos.0 as isize + dx;
            let y = mouse_pos.1 as isize + dy;

            if x >= 0 && x < self.width.get() as isize && y >= 0 && y < self.height.get() as isize {
                let index = (y * self.width.get() as isize + x) as usize;
                if self.is_empty(index) {
                    self.set_pixel(index, color);
                    placed_pixels = true;
                }
            }
        }
        placed_pixels
    }

    pub fn resize(&mut self, width: NonZeroUsize, height: NonZeroUsize) {
        resize::smart_resize(
            &mut self.colors,
            (self.width.into(), self.height.into()),
            (width.into(), height.into()),
        );
        self.width = width;
        self.height = height;
    }

    pub(crate) fn set_radius(&mut self, r: f64) {
        self.radius = r;
    }
}
