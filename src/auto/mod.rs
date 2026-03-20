use crate::gradient::Gradient;

mod layers;
mod mountains;
pub mod utils;

trait Strategy {
    fn starting_pos(&self) -> (f32, f32);
    fn should_spawn(&self) -> bool;
    fn choose_new_color(&mut self, gradient: &Gradient) -> bool;
    fn tick(&mut self, mouse_pos: &mut (f32, f32));
}

pub struct Auto {
    pub enabled: bool,
    strategy: Box<dyn Strategy>,
    mouse_pos: (f32, f32),
}

impl Auto {
    pub fn new() -> Self {
        // let strategy = Box::new(layers::Layers::new());
        let strategy = Box::new(mountains::Mountains::new());
        let mouse_pos = strategy.starting_pos();

        Self {
            enabled: false,
            strategy,
            mouse_pos,
        }
    }

    pub fn flip(&mut self) {
        if !self.enabled {
            *self = Auto::new();
            self.enabled = true;
        } else {
            self.enabled = false;
        }
    }

    pub fn mouse_pos(&self) -> (f32, f32) {
        self.mouse_pos
    }

    pub fn should_spawn(&self) -> bool {
        self.enabled && self.strategy.should_spawn()
    }

    pub fn choose_new_color(&mut self, gradient: &Gradient) -> bool {
        self.enabled && self.strategy.choose_new_color(gradient)
    }

    pub fn tick(&mut self) {
        if self.enabled {
            self.strategy.tick(&mut self.mouse_pos);
        }
    }
}
