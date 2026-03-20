use crate::{
    auto::{utils::is_nice_color, Strategy},
    gradient::Gradient,
};

const MAX_TRIES: u8 = 5;
enum Intention {
    ChooseColor { tries: u8 },
    DrawLayer { left_to_right: bool },
}

pub struct Layers {
    intention: Intention,
    next_l_to_r: bool,
}
impl Layers {
    pub(crate) fn new() -> Self {
        Self {
            intention: Intention::ChooseColor { tries: 0 },
            next_l_to_r: true,
        }
    }
}

impl Strategy for Layers {
    fn starting_pos(&self) -> (f32, f32) {
        (0.0, 0.001)
    }

    fn should_spawn(&self) -> bool {
        matches!(self.intention, Intention::DrawLayer { .. })
    }

    fn choose_new_color(&mut self, gradient: &Gradient) -> bool {
        let Intention::ChooseColor { tries } = self.intention else {
            return false;
        };

        if tries != 0 && (tries > MAX_TRIES || is_nice_color(gradient.peek_color())) {
            self.intention = Intention::DrawLayer {
                left_to_right: self.next_l_to_r,
            };
            self.next_l_to_r = !self.next_l_to_r;
            false
        } else {
            self.intention = Intention::ChooseColor { tries: tries + 1 };
            true
        }
    }

    fn tick(&mut self, mouse_pos: &mut (f32, f32)) {
        let Intention::DrawLayer { left_to_right } = self.intention else {
            return;
        };
        let change = 0.005 * (if left_to_right { 1.0 } else { -1.0 });
        mouse_pos.0 += change;

        if mouse_pos.0 >= 1.0 || mouse_pos.0 <= 0.0 {
            self.intention = Intention::ChooseColor { tries: 0 };
        }
    }
}
