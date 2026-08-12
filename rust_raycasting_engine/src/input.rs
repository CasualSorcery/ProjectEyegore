pub struct InputState {
    pub last_mouse_x: Option<f32>,
}

impl InputState {
    pub fn new() -> Self {
        Self { last_mouse_x: None }
    }
}