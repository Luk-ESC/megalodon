use std::num::NonZeroU16;

use fastrand::Rng;

pub type Steps = NonZeroU16;
#[derive(Clone)]
pub(crate) struct Gradient {
    start: (u8, u8, u8),
    end: (u8, u8, u8),
    i: u32,
    step: (f32, f32, f32),
    cur_step: (f32, f32, f32),
}

fn calculate_step(start: (u8, u8, u8), end: (u8, u8, u8), steps: Steps) -> (f32, f32, f32) {
    (
        ((end.0 - start.0) as f32 / steps.get() as f32),
        ((end.1 - start.1) as f32 / steps.get() as f32),
        ((end.2 - start.2) as f32 / steps.get() as f32),
    )
}

impl Gradient {
    pub fn new(rng: &mut Rng, steps: Steps) -> Self {
        let n1: (u8, u8, u8) = (rng.u8(..), rng.u8(..), rng.u8(..));
        let n2: (u8, u8, u8) = (rng.u8(..), rng.u8(..), rng.u8(..));

        let start = (n1.0.min(n2.0), n1.1.min(n2.1), n1.2.min(n2.2));

        let end = (n1.0.max(n2.0), n1.1.max(n2.1), n1.2.max(n2.2));
        let cur_step = (start.0 as f32, start.1 as f32, start.2 as f32);

        let step = calculate_step(start, end, steps);
        Self {
            start,
            end,
            i: 0,
            step,
            cur_step,
        }
    }

    pub fn peek_color(&self) -> u32 {
        let color = (
            self.cur_step.0 as u8,
            self.cur_step.1 as u8,
            self.cur_step.2 as u8,
        );

        ((color.0 as u32) << 16) | ((color.1 as u32) << 8) | color.2 as u32
    }

    pub fn next_color(&mut self) -> u32 {
        let mut color = (
            self.cur_step.0 as u8,
            self.cur_step.1 as u8,
            self.cur_step.2 as u8,
        );

        if color >= self.end {
            color = self.end;
            self.cur_step = (
                self.start.0 as f32,
                self.start.1 as f32,
                self.start.2 as f32,
            );
            self.i = 0;
        } else {
            self.i += 1;

            self.cur_step.0 += self.step.0;
            self.cur_step.1 += self.step.1;
            self.cur_step.2 += self.step.2;
        }

        ((color.0 as u32) << 16) | ((color.1 as u32) << 8) | color.2 as u32
    }
}
