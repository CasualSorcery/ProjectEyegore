use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CartesianPos {
    pub x: f64,
    pub y: f64,
}
