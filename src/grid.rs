use fastrand::Rng;

use crate::{radii::RadiusId, resize, DEFAULT_HEIGHT, DEFAULT_WIDTH};

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
    width: u16,
    height: u16,
    radius: f64,
    pub colors: Vec<u32>,
    rng: Rng,
    highest_row: u16,
}

pub const EMPTY: u32 = 0xFFE0FFFE;

impl Grid {
    pub fn new() -> Self {
        Self {
            width: DEFAULT_WIDTH as u16,
            height: DEFAULT_HEIGHT as u16,
            radius: RadiusId::default().get(),
            colors: vec![EMPTY; DEFAULT_WIDTH * DEFAULT_HEIGHT],
            rng: Rng::new(),
            highest_row: DEFAULT_HEIGHT as u16 - 1,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut updated = false;

        for row in (self.highest_row..=self.height - 2).rev() {
            let mut updated_this_row = false;
            let direction = if self.rng.bool() { 1i16 } else { -1 };

            let mut column = if direction > 0 { 0 } else { self.width - 1 };

            let limit = if direction > 0 { self.width - 1 } else { 0 };

            loop {
                let i = row as u32 * self.width as u32 + column as u32;

                if !self.is_empty(i) {
                    let moved_down = self.update_pixel(i, column);
                    updated_this_row |= moved_down;
                }

                if column == limit {
                    break;
                }

                column += direction as u16;
            }

            updated |= updated_this_row;
            if !updated_this_row && row == self.highest_row {
                // move highest row down
                self.highest_row = self.height.min(self.highest_row + 1);
            } else if updated_this_row {
                // check row above this one again
                self.highest_row = self.highest_row.min(row - 1);
            }
        }

        updated
    }

    fn update_pixel(&mut self, i: u32, column: u16) -> bool {
        let below = i + self.width as u32;
        let below_left = below - 1;
        let below_right = below + 1;

        // If there are no pixels below, move it down.
        if self.is_empty(below) {
            self.move_(i, below);
            return true;
        } else if column != 0 && self.is_empty(below_left) {
            self.move_(i, below_left);
            return true;
        } else if column != self.width - 1 && self.is_empty(below_right) {
            self.move_(i, below_right);
            return true;
        }
        false
    }

    pub fn clear(&mut self) {
        self.colors.fill(EMPTY);
        self.highest_row = self.height - 1;
    }

    fn move_(&mut self, a: u32, b: u32) {
        self.colors[b as usize] = self.colors[a as usize];
        self.colors[a as usize] = EMPTY;
    }

    pub fn set_pixel(&mut self, a: u32, color: u32) {
        assert!(self.is_empty(a));

        self.colors[a as usize] = color;
    }

    pub fn is_empty(&self, a: u32) -> bool {
        self.colors[a as usize] == EMPTY
    }

    pub fn spawn(&mut self, mouse_pos: (u16, u16), color: u32) -> bool {
        let mut placed_pixels = false;
        for (dx, dy) in circle_offsets(self.radius) {
            let x = mouse_pos.0 as isize + dx;
            let y = mouse_pos.1 as isize + dy;

            if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                self.highest_row = self.highest_row.min(y as u16);
                let index = (y * self.width as isize + x) as u32;
                if self.is_empty(index) {
                    self.set_pixel(index, color);
                    placed_pixels = true;
                }
            }
        }
        placed_pixels
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        resize::smart_resize(
            &mut self.colors,
            (self.width as usize, self.height as usize),
            (width as usize, height as usize),
        );
        self.width = width;
        self.height = height;
        self.highest_row = 0;
    }

    pub(crate) fn set_radius(&mut self, r: f64) {
        self.radius = r;
    }
}
