#![allow(dead_code)]
#![allow(unused_variables)]
use crate::core::config::{GameConfig, Level};
use crate::world::entity::{EnemyState, Entity, EntityType};
use crate::world::map::CartesianPos;
use serde::Deserialize;
use serde_json::Value;
use std::fs;

// Tiled intermediate structs
// needed for appropriate JSON Deserializing
#[derive(Deserialize, Debug)]
struct TiledProperty {
    name: String,
    value: Value,
}

#[derive(Deserialize, Debug)]
struct TiledObject {
    name: String,
    x: f64,
    y: f64,
    #[serde(rename = "type")]
    obj_type: String,
    properties: Option<Vec<TiledProperty>>,
}

#[derive(Deserialize, Debug)]
struct TiledLayer {
    name: String,
    #[serde(rename = "type")]
    layer_type: String,
    data: Option<Vec<u8>>,
    objects: Option<Vec<TiledObject>>,
    properties: Option<Vec<TiledProperty>>,
}

#[derive(Deserialize, Debug)]
struct TiledTile {
    id: usize,
    image: String,
}

#[derive(Deserialize, Debug)]
struct TiledTileset {
    tiles: Option<Vec<TiledTile>>,
}

#[derive(Deserialize, Debug)]
struct TiledMap {
    width: usize,
    height: usize,
    properties: Option<Vec<TiledProperty>>,
    layers: Vec<TiledLayer>,
    tilesets: Vec<TiledTileset>,
}

/// Helper function to get the property Value from the JSON structure
fn get_property_value(props: &Option<Vec<TiledProperty>>, name: &str) -> Option<Value> {
    props
        .as_ref()?
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.value.clone())
}

pub fn load_from_json(filepath: &str) -> GameConfig {
    let file_content = fs::read_to_string(filepath).expect("Failed to read JSON file");

    // get all the raw JSON properties into a TiledMap struct
    let tiled_map: TiledMap =
        serde_json::from_str(&file_content).expect("Failed to parse Tiled JSON");

    // GameConfig main struct properties
    let game_name = get_property_value(&tiled_map.properties, "game_name")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .expect("Missing 'game_name' property in Tiled map");

    let scr_width = get_property_value(&tiled_map.properties, "scr_width")
        .and_then(|v| v.as_u64())
        .expect("Missing 'scr_width' property in Tiled map") as usize;

    let scr_height = get_property_value(&tiled_map.properties, "scr_height")
        .and_then(|v| v.as_u64())
        .expect("Missing 'scr_height' property in Tiled map") as usize;

    let target_fps = get_property_value(&tiled_map.properties, "target_fps")
        .and_then(|v| v.as_u64())
        .expect("Missing 'target_fps' property in Tiled map") as usize;

    let fullscreen = get_property_value(&tiled_map.properties, "fullscreen")
        .and_then(|v| v.as_bool())
        .expect("Missing 'fullscreen' property in Tiled map");

    // Level struct properties
    let mut level_name = String::new();
    let mut wall_map = Vec::new();
    let mut floor_map = Vec::new();
    let mut ceil_map = Vec::new();
    let mut entities = Vec::new();
    let mut player_start_x = 0.0;
    let mut player_start_y = 0.0;
    let mut max_fog_distance = 10.0;

    for layer in tiled_map.layers {
        match layer.name.as_str() {
            "wall_map" => {
                wall_map = layer.data.unwrap_or_default();
                if let Some(fog_val) = get_property_value(&layer.properties, "max_fog_distance") {
                    max_fog_distance = fog_val.as_f64().unwrap_or(10.0);
                }
                if let Some(level_name_val) = get_property_value(&layer.properties, "level_name") {
                    level_name = level_name_val.to_string();
                }
            }
            "floor_map" => floor_map = layer.data.unwrap_or_default(),
            "ceil_map" => ceil_map = layer.data.unwrap_or_default(),
            "entities" => {
                if let Some(objects) = layer.objects {
                    for obj in objects {
                        if obj.name == "player_start" {
                            player_start_x = obj.x;
                            player_start_y = obj.y;
                            continue;
                        }

                        let texture = get_property_value(&obj.properties, "texture")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize;

                        let entity_type = match obj.obj_type.as_str() {
                            "Enemy" => {
                                let hp = get_property_value(&obj.properties, "hp")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(100.0)
                                    as f32;
                                let speed = get_property_value(&obj.properties, "speed")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(1.0);
                                let state_str = get_property_value(&obj.properties, "state")
                                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                                    .unwrap_or_default();

                                let state = match state_str.as_str() {
                                    "Chasing" => EnemyState::Chasing,
                                    "Dead" => EnemyState::Dead,
                                    _ => EnemyState::Idle,
                                };

                                EntityType::Enemy { hp, state, speed }
                            }
                            _ => EntityType::Prop,
                        };

                        entities.push(Entity {
                            pos: CartesianPos { x: obj.x, y: obj.y },
                            texture,
                            scale_x: 1.0,
                            scale_y: 1.0,
                            entity_type,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    let mut textures = Vec::new();
    for tileset in tiled_map.tilesets {
        if let Some(tiles) = tileset.tiles {
            // Sort by id just to keep the order safe
            let mut sorted_tiles = tiles;
            sorted_tiles.sort_by_key(|t| t.id);
            for tile in sorted_tiles {
                textures.push(tile.image);
            }
        }
    }

    let level = Level {
        name: level_name,
        map_width: tiled_map.width,
        map_height: tiled_map.height,
        wall_map,
        floor_map,
        ceil_map,
        player_start_x,
        player_start_y,
        max_fog_distance,
        textures,
        entities,
    };

    // gather all and return GameConfig
    GameConfig {
        name: game_name,
        scr_width,
        scr_height,
        target_fps,
        fullscreen,
        levels: vec![level],
    }
}