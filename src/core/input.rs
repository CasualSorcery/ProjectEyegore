use std::collections::HashSet;
use std::time::Instant;
use winit::keyboard::PhysicalKey;

// TODO: Add actual in-game configuration

/// Main Mouse and Keyboard input structure
pub struct InputState {
    /// hash map of all the physical keys readings held at the moment
    pub keys_held: HashSet<PhysicalKey>,
    /// mouse x and y directionals
    pub mouse_dx: f64,
    pub mouse_dy: f64,
    /// shoot logic variables
    pub left_mouse_down: bool,
    pub last_shot_time: Instant,
}
impl InputState {
    pub fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            left_mouse_down: false,
            last_shot_time: Instant::now(),
        }
    }
    pub fn is_key_down(&self, key: PhysicalKey) -> bool {
        self.keys_held.contains(&key)
    }
}
