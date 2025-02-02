use std::num::NonZeroU8;

use rand::Rng;
#[derive(Clone)]
pub(crate) struct Gradient {
    start: (u8, u8, u8),
    end: (u8, u8, u8),
    i: u32,
    step: (f32, f32, f32),
}

fn calculate_step(start: (u8, u8, u8), end: (u8, u8, u8), steps: NonZeroU8) -> (f32, f32, f32) {
    (
        ((end.0 - start.0) as f32 / steps.get() as f32),
        ((end.1 - start.1) as f32 / steps.get() as f32),
        ((end.2 - start.2) as f32 / steps.get() as f32),
    )
}

impl Gradient {
    pub fn new(rng: &mut impl Rng, steps: NonZeroU8) -> Self {
        let n1: (u8, u8, u8) = rng.gen();
        let n2: (u8, u8, u8) = rng.gen();

        let start = (n1.0.min(n2.0), n1.1.min(n2.1), n1.2.min(n2.2));

        let end = (n1.0.max(n2.0), n1.1.max(n2.1), n1.2.max(n2.2));

        let step = calculate_step(start, end, steps);
        Self {
            start,
            end,
            i: 0,
            step,
        }
    }

    pub fn set_steps(&mut self, steps: NonZeroU8) {
        self.step = calculate_step(self.start, self.end, steps);
        self.i = 0;
    }

    pub fn peek_color(&self) -> u32 {
        let color = (
            self.start.0 as f32 + self.step.0 * self.i as f32,
            self.start.1 as f32 + self.step.1 * self.i as f32,
            self.start.2 as f32 + self.step.2 * self.i as f32,
        );
        let color = (
            color.0.round() as u8,
            color.1.round() as u8,
            color.2.round() as u8,
        );

        (color.0 as u32) << 16 | (color.1 as u32) << 8 | color.2 as u32
    }

    pub fn next_color(&mut self) -> u32 {
        let color = (
            self.start.0 as f32 + self.step.0 * self.i as f32,
            self.start.1 as f32 + self.step.1 * self.i as f32,
            self.start.2 as f32 + self.step.2 * self.i as f32,
        );
        let mut color = (
            color.0.round() as u8,
            color.1.round() as u8,
            color.2.round() as u8,
        );

        if color >= self.end {
            color = self.end;
            self.i = 0;
        } else {
            self.i += 1;
        }

        (color.0 as u32) << 16 | (color.1 as u32) << 8 | color.2 as u32
    }
}
