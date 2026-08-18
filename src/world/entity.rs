#![allow(dead_code)]
#![allow(unused_variables)]
use crate::world::map::CartesianPos;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum EnemyState {
    Idle,
    Chasing,
    Dead,
}
#[derive(Deserialize, Debug)]
pub enum EffectType {
    Heal,
    Damage,
    Ammo,
}
#[derive(Deserialize, Debug)]
pub enum EntityType {
    Prop,
    Enemy {
        hp: f32,
        state: EnemyState,
        speed: f64,
    },
    Pickup {
        effect: EffectType,
        amount: i32,
    },
}
#[derive(Deserialize, Debug)]
pub struct Entity {
    pub pos: CartesianPos,

    pub texture: usize,
    pub scale_x: f64,
    pub scale_y: f64,

    pub entity_type: EntityType,
}
impl Entity {
    //noinspection RsExternalLinter
    pub fn new_prop(x: f64, y: f64, texture: usize) -> Self {
        Self {
            // create new prop entity
            pos: CartesianPos { x, y },
            texture,
            scale_x: 1.0,
            scale_y: 1.0,
            entity_type: EntityType::Prop,
        }
    }
    pub fn new_enemy(x: f64, y: f64, texture: usize, hp: f32, speed: f64) -> Self {
        Self {
            // create new enemy entity
            pos: CartesianPos { x, y },
            texture,
            scale_x: 1.0,
            scale_y: 1.0,
            entity_type: EntityType::Enemy {
                hp,
                state: EnemyState::Idle,
                speed,
            },
        }
    }
    pub fn new_pickup(x: f64, y: f64, texture: usize, effect: EffectType, amount: i32) -> Self {
        // create new pickup entity
        Self {
            pos: CartesianPos { x, y },
            texture,
            scale_x: 0.5,
            scale_y: 0.5,
            entity_type: EntityType::Pickup { effect, amount },
        }
    }
    pub fn update(
        &mut self,
        player_pos: &CartesianPos,
        frame_time: f64,
        map: &[u8],
        map_width: usize,
    ) {
        // handle entity behavior
        match &mut self.entity_type {
            EntityType::Prop => {}

            EntityType::Pickup {
                effect: _,
                amount: _,
            } => {}

            EntityType::Enemy { hp, state, speed } => {
                // if HP drops below 0 and not dead, die
                if *hp <= 0.0 && !matches!(state, EnemyState::Dead) {
                    *state = EnemyState::Dead;
                    self.texture = 8; // dead body texture
                }

                match state {
                    EnemyState::Idle => {
                        let dist_sq = (player_pos.x - self.pos.x).powi(2)
                            + (player_pos.y - self.pos.y).powi(2);
                        if dist_sq < 64.0 {
                            *state = EnemyState::Chasing;
                        }
                    }
                    EnemyState::Chasing => {
                        let dx = player_pos.x - self.pos.x;
                        let dy = player_pos.y - self.pos.y;
                        let dist = (dx.powi(2) + dy.powi(2)).sqrt();

                        if dist > 0.5 {
                            let move_step = *speed * frame_time;

                            let next_x = self.pos.x + (dx / dist) * move_step;
                            let next_y = self.pos.y + (dy / dist) * move_step;

                            let index_x = (next_x as usize) * map_width + (self.pos.y as usize);

                            // X Collision Check
                            if *map.get(index_x).unwrap_or(&1) == 0 {
                                self.pos.x = next_x;
                            }

                            // Y Collision Check
                            let index_y = (self.pos.x as usize) * map_width + (next_y as usize);
                            if *map.get(index_y).unwrap_or(&1) == 0 {
                                self.pos.y = next_y;
                            }
                        }
                    }
                    EnemyState::Dead => {}
                }
            }
        }
    }
}
