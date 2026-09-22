#![allow(dead_code)]
#![allow(unused_variables)]

use crate::core::json_parser::load_from_json;
use crate::world::entity::Entity;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct GameConfig {
    pub name: String,
    pub scr_width: usize,
    pub scr_height: usize,
    pub target_fps: usize,
    pub fullscreen: bool,
    pub levels: Vec<Level>,
}
#[derive(Deserialize)]
pub struct Level {
    pub name: String,
    pub map_width: usize,
    pub map_height: usize,
    pub wall_map: Vec<u8>,
    pub floor_map: Vec<u8>,
    pub ceil_map: Vec<u8>,
    pub player_start_x: f64,
    pub player_start_y: f64,
    pub max_fog_distance: f64,
    pub textures: Vec<String>,
    pub entities: Vec<Entity>,
}

pub fn load_config_from_file(filepath: &str) -> GameConfig {
    // transforms into Path for the sole usage of .extension()
    let path = Path::new(filepath);

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ron") => load_from_ron(filepath),
        Some("json") | Some("tmj") => load_from_json(filepath),
        _ => panic!("Unsupported file extension: {}", filepath),
    }
}
pub fn load_from_ron(filepath: &str) -> GameConfig {
    // tries to read the ron file
    let file_content = std::fs::read_to_string(filepath).expect("Failed to read file");
    // tries to serialize the ron file into the GameConfig Struct
    ron::from_str(&file_content).expect("Failed to parse RON") // returning the result as GameConfig
}