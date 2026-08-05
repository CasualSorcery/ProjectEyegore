use crate::map::CartesianPos;

pub struct Player {
    pub position: CartesianPos,
    pub direction: CartesianPos,
    pub plane: CartesianPos,
    pub move_speed: f64,
    pub rotation_speed: f64,
}