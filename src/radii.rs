static RADII: &[f64] = &[1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0];

#[derive(Clone, Copy)]
pub struct RadiusId(u8);

impl Default for RadiusId {
    fn default() -> Self {
        RadiusId(3)
    }
}

impl RadiusId {
    pub fn next_bigger(&mut self) {
        self.0 = (self.0 + 1).min(RADII.len() as u8 - 1)
    }

    pub fn next_smaller(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    pub fn get(&self) -> f64 {
        RADII[self.0 as usize]
    }
}
