use std::time::Instant;

pub struct InputState {
    pub last_mouse_x: Option<f32>,
    pub last_shot_time: Instant,
}
impl InputState {
    pub fn new() -> Self {
        Self {
            last_mouse_x: None,
            last_shot_time: Instant::now(),
        }
    }
}
