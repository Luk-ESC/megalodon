use crate::{
    auto::{utils::is_nice_color, Strategy},
    gradient::Gradient,
};

const MAX_TRIES: u8 = 5;
enum Intention {
    ChooseColor {
        tries: u8,
    },
    DrawMountain {
        x_pos: f32,
        ticks_ttl: u32,
        width: f32,
        velocity: f32,
    },
}

pub struct Mountains {
    intention: Intention,
    next_l_to_r: bool,
    started_drawing: bool,
}

impl Mountains {
    pub fn new() -> Self {
        Self {
            intention: Intention::ChooseColor { tries: 0 },
            next_l_to_r: true,
            started_drawing: false,
        }
    }
}

impl Strategy for Mountains {
    fn starting_pos(&self) -> (f32, f32) {
        (0.0, 0.001)
    }

    fn should_spawn(&self) -> bool {
        matches!(self.intention, Intention::DrawMountain { .. }) && !self.started_drawing
    }

    fn choose_new_color(&mut self, gradient: &Gradient) -> bool {
        let Intention::ChooseColor { tries } = self.intention else {
            return false;
        };

        if tries != 0 && (tries > MAX_TRIES || is_nice_color(gradient.peek_color())) {
            let x_pos = fastrand::f32();
            let max_width = (1.0 - x_pos).abs().min(0.2);
            let width = fastrand::f32() * max_width;
            let ticks_ttl = fastrand::u32(200..1000);
            let velocity = width * 4.0 / (ticks_ttl as f32);

            self.intention = Intention::DrawMountain {
                x_pos,
                ticks_ttl,
                width,
                velocity,
            };
            self.started_drawing = true;
            false
        } else {
            self.intention = Intention::ChooseColor { tries: tries + 1 };
            true
        }
    }

    fn tick(&mut self, mouse_pos: &mut (f32, f32)) {
        let Intention::DrawMountain {
            x_pos,
            ticks_ttl,
            width,
            velocity,
        } = self.intention
        else {
            return;
        };

        if self.started_drawing {
            self.started_drawing = false;
            mouse_pos.0 = x_pos;
        }

        let change = velocity * (if self.next_l_to_r { 1.0 } else { -1.0 });
        mouse_pos.0 += change;

        if mouse_pos.0 >= x_pos + width || mouse_pos.0 <= x_pos - width {
            self.next_l_to_r = !self.next_l_to_r;
        }

        if ticks_ttl == 0 {
            self.next_l_to_r = !self.next_l_to_r;
            self.intention = Intention::ChooseColor { tries: 0 };
        } else {
            self.intention = Intention::DrawMountain {
                x_pos,
                ticks_ttl: ticks_ttl - 1,
                width,
                velocity,
            };
        }
    }
}
