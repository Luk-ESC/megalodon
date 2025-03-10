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
    #[cfg(debug_assertions)]
    pub checked: Vec<u32>,
    rng: Rng,
    highest_row: u16,
    lowest_row: u16,
    left_skip: u16,
    right_skip: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    None = 0,
    Down = 1,
    DownLeft = 2,
    DownRight = 3,
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
            #[cfg(debug_assertions)]
            checked: vec![EMPTY; DEFAULT_WIDTH * DEFAULT_HEIGHT],
            highest_row: DEFAULT_HEIGHT as u16 - 1,
            lowest_row: 0,
            left_skip: DEFAULT_WIDTH as u16 - 1,
            right_skip: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut updated = false;

        #[cfg(debug_assertions)]
        self.checked.fill(EMPTY);
        let mut lowest_row = 0;
        let mut most_left = self.width - 1;
        let mut most_right = 0;
        for row in (self.highest_row..=self.lowest_row).rev() {
            let mut updated_this_row = false;
            let direction = if self.rng.bool() { 1i16 } else { -1 };

            let mut column = if direction > 0 {
                self.left_skip
            } else {
                self.right_skip
            };

            let limit = if direction > 0 {
                self.right_skip
            } else {
                self.left_skip
            };

            loop {
                let i = row as u32 * self.width as u32 + column as u32;

                if !self.is_empty(i) {
                    let moved = self.update_pixel(i, column);
                    updated_this_row |= moved != Direction::None;

                    if moved == Direction::DownLeft {
                        // Moved left, column to the left has a pixel
                        most_left = most_left.min(column - 1);
                    } else if moved == Direction::DownRight {
                        // Moved right, column to the right has a pixel
                        most_right = most_right.max(column + 1);
                    } else {
                        // Stayed in this colum.
                        most_left = most_left.min(column);
                        most_right = most_right.max(column);
                    }
                }

                if column == limit {
                    break;
                }

                column = column.wrapping_add(direction as u16);
            }

            updated |= updated_this_row;
            if !updated_this_row && row == self.highest_row {
                // move highest row down
                self.highest_row = self.height.min(self.highest_row + 1);
            } else if updated_this_row {
                // check row above this one again
                self.highest_row = self.highest_row.min(row - 1);
                lowest_row = lowest_row.max(row + 1);
            }
        }

        self.lowest_row = lowest_row.min(self.height - 2);
        self.left_skip = most_left;
        self.right_skip = most_right.min(self.width - 1);

        updated
    }

    fn update_pixel(&mut self, i: u32, column: u16) -> Direction {
        let below = i + self.width as u32;
        let below_left = below - 1;
        let below_right = below + 1;

        // If there are no pixels below, move it down.
        if self.is_empty(below) {
            self.move_(i, below);
            return Direction::Down;
        } else if column != 0 && self.is_empty(below_left) {
            self.move_(i, below_left);
            return Direction::DownLeft;
        } else if column != self.width - 1 && self.is_empty(below_right) {
            self.move_(i, below_right);
            return Direction::DownRight;
        }
        Direction::None
    }

    pub fn clear(&mut self) {
        self.colors.fill(EMPTY);
        #[cfg(debug_assertions)]
        self.checked.fill(EMPTY);
        self.highest_row = self.height - 1;
        self.lowest_row = 0;
        self.left_skip = self.width - 1;
        self.right_skip = 0;
    }

    fn move_(&mut self, a: u32, b: u32) {
        self.colors[b as usize] = self.colors[a as usize];
        self.colors[a as usize] = EMPTY;
    }

    pub fn set_pixel(&mut self, a: u32, color: u32) {
        assert!(self.is_empty(a));

        self.colors[a as usize] = color;
    }

    pub fn is_empty(&mut self, a: u32) -> bool {
        let ret = self.colors[a as usize] == EMPTY;
        #[cfg(debug_assertions)]
        if ret {
            self.checked[a as usize] = 0xFFFF0000;
        } else {
            self.checked[a as usize] = 0xFF00FF00;
        }
        ret
    }

    pub fn spawn(&mut self, mouse_pos: (u16, u16), color: u32) -> bool {
        let mut placed_pixels = false;
        for (dx, dy) in circle_offsets(self.radius) {
            let x = mouse_pos.0 as isize + dx;
            let y = mouse_pos.1 as isize + dy;

            if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                self.highest_row = self.highest_row.min(y as u16);
                self.lowest_row = self.lowest_row.max(y as u16).min(self.height - 2);
                self.left_skip = self.left_skip.min(x as u16);
                self.right_skip = self.right_skip.max(x as u16).min(self.width - 1);
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
        #[cfg(debug_assertions)]
        self.checked
            .resize(width as usize * height as usize, 0xFFFF0000);
        self.width = width;
        self.height = height;
        self.highest_row = 0;
        self.lowest_row = self.height - 2;
        self.left_skip = 0;
        self.right_skip = self.width - 1;
    }

    pub(crate) fn set_radius(&mut self, r: f64) {
        self.radius = r;
    }
}
