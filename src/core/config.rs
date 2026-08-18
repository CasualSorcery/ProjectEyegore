use crate::world::entity::Entity;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameConfig {
    pub name: String,
    pub scr_width: usize,
    pub scr_height: usize,
    pub fullscreen: bool,
    pub levels: Vec<Level>,
}
#[derive(Deserialize)]
pub struct Level {
    pub _name: String,
    pub map_width: usize,
    pub map_height: usize,
    pub map: Vec<u8>,
    pub player_start_x: f64,
    pub player_start_y: f64,
    pub max_fog_distance: f64,
    pub floor_tex_idx: usize,
    pub ceil_tex_idx: usize,
    pub textures: Vec<String>,
    pub entities: Vec<Entity>,
}
pub fn load_config(filepath: &str) -> GameConfig {
    // tries to read the ron file
    let file_content = std::fs::read_to_string(filepath).expect("Failed to read file");
    // tries to serialize the ron file into the GameConfig Struct
    let config = ron::from_str(&file_content).expect("Failed to parse RON");

    // returns the config
    config
}
