#![allow(dead_code)]
#![allow(unused_variables)]
use super::Engine;
use crate::utils::helpers::load_texture;
use crate::world::map::CartesianPos;
use std::time::Instant;
use winit::keyboard::{KeyCode, PhysicalKey};

impl Engine {
    // helper function, gets the tile according to the coordinates

    /// Gets a single, specific tile index in the 1d map array,
    /// according to the provided 2d coordinates inputted in the current level.
    ///
    ///
    /// # Arguments
    ///
    /// * `x` - the x coordinate of the tile.
    /// * `y` - the y coordinate of the tile.
    ///
    /// # Returns
    ///
    /// * the `u8` address of the tile on the 1d array.
    #[inline(always)]
    pub(crate) fn get_tile(&self, x: usize, y: usize) -> u8 {
        let level = &self.config.levels[self.current_level_idx];

        // out of bounds check
        if x >= level.map_width || y >= level.map_height { return 1; }

        // converts 2d array math to the 1d array index
        let index = y * level.map_width + x;

        *level.wall_map.get(index).unwrap_or(&1)
    }

    // input handling, be it from movement or interaction

    /// Handles the player's inputs
    ///
    /// # Arguments
    ///
    /// * `frame_time` - Delta time in seconds since last frame.
    pub(crate) fn handle_input(&mut self, frame_time: f64) {
        // ------------------------------------------------------------
        // ui controls
        // ------------------------------------------------------------

        // if paused, no input accepted
        if self.is_paused {
            return;
        }

        // ------------------------------------------------------------
        // movement and vision control
        // ------------------------------------------------------------

        let move_step = frame_time * self.player.move_speed;

        let right_dir_x = -self.player.direction.y;
        let right_dir_y = self.player.direction.x;

        let mut input_x = 0.0;
        let mut input_y = 0.0;

        // spatial movement UP DOWN RIGHT LEFT
        if self.input.is_key_down(PhysicalKey::Code(KeyCode::KeyW)) {
            input_y += 1.0;
        }
        if self.input.is_key_down(PhysicalKey::Code(KeyCode::KeyS)) {
            input_y -= 1.0;
        }
        if self.input.is_key_down(PhysicalKey::Code(KeyCode::KeyD)) {
            input_x -= 1.0;
        }
        if self.input.is_key_down(PhysicalKey::Code(KeyCode::KeyA)) {
            input_x += 1.0;
        }

        // normalized movement handling
        if input_x != 0.0 || input_y != 0.0 {
            if input_x != 0.0 && input_y != 0.0 {
                // famous fast inverse sqrt magic address
                let inv_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;
                input_x *= inv_sqrt2;
                input_y *= inv_sqrt2;
            }

            let move_vec_x =
                (self.player.direction.x * input_y + right_dir_x * input_x) * move_step;
            let move_vec_y =
                (self.player.direction.y * input_y + right_dir_y * input_x) * move_step;

            let next_x = self.player.position.x + move_vec_x;
            let next_y = self.player.position.y + move_vec_y;

            // collision check
            if next_x >= 0.0 && self.get_tile(next_x as usize, self.player.position.y as usize) == 0 {
                self.player.position.x = next_x;
            }
            if next_y >= 0.0 && self.get_tile(self.player.position.x as usize, next_y as usize) == 0 {
                self.player.position.y = next_y;
            }
        }

        // raw input mouse handling
        let mouse_sensitivity = 0.003;
        let mouse_delta_x = self.input.mouse_dx;

        if mouse_delta_x != 0.0 {
            let rot_step = -mouse_delta_x * mouse_sensitivity;
            let cos_rot = rot_step.cos();
            let sin_rot = rot_step.sin();

            let old_dir_x = self.player.direction.x;
            self.player.direction.x =
                self.player.direction.x * cos_rot - self.player.direction.y * sin_rot;
            self.player.direction.y = old_dir_x * sin_rot + self.player.direction.y * cos_rot;

            let old_plane_x = self.player.plane.x;
            self.player.plane.x = self.player.plane.x * cos_rot - self.player.plane.y * sin_rot;
            self.player.plane.y = old_plane_x * sin_rot + self.player.plane.y * cos_rot;
        }

        // reset mouse input
        self.input.mouse_dx = 0.0;
        self.input.mouse_dy = 0.0;

        // ------------------------------------------------------------
        // interactions
        // ------------------------------------------------------------

        // shoot/interact logic (actually just left mouse button handling)
        if self.input.left_mouse_down {
            let now = Instant::now();

            if now.duration_since(self.input.last_shot_time).as_secs_f64() > 0.3 {
                let mut damage_to_deal = 0.0;

                if let Some(crate::world::player::Items::Weapon {
                                damage,
                                ammo,
                                name: _,
                            }) = self.player.inventory.get_current_wpn_mut()
                    && *ammo > 0
                {
                    *ammo -= 1;
                    damage_to_deal = *damage;
                    self.input.last_shot_time = now;
                }

                if damage_to_deal > 0.0 {
                    let mut hit_index: Option<usize> = None;
                    let mut closest_dist = 1e30;

                    let level = &mut self.config.levels[self.current_level_idx];

                    for (i, entity) in level.entities.iter().enumerate() {
                        if let crate::world::entity::EntityType::Enemy { hp, .. } =
                            &entity.entity_type
                        {
                            if *hp <= 0.0 {
                                continue;
                            }

                            let dx = entity.pos.x - self.player.position.x;
                            let dy = entity.pos.y - self.player.position.y;
                            let dist = (dx.powi(2) + dy.powi(2)).sqrt();

                            let angle_to_enemy = dy.atan2(dx);
                            let player_angle =
                                self.player.direction.y.atan2(self.player.direction.x);

                            let mut angle_diff = angle_to_enemy - player_angle;
                            while angle_diff > std::f64::consts::PI {
                                angle_diff -= 2.0 * std::f64::consts::PI;
                            }
                            while angle_diff < -std::f64::consts::PI {
                                angle_diff += 2.0 * std::f64::consts::PI;
                            }

                            if angle_diff.abs() < 0.2 && dist < closest_dist {
                                closest_dist = dist;
                                hit_index = Some(i);
                            }
                        }
                    }

                    if let Some(idx) = hit_index
                        && let crate::world::entity::EntityType::Enemy { ref mut hp, .. } =
                        level.entities[idx].entity_type
                    {
                        *hp -= damage_to_deal;
                    }
                }
            }
        }
    }

    // update all entities states

    /// Updates all entities inside the current level.
    ///
    /// # Arguments
    ///
    /// * `frame_time` - Delta time in seconds since last frame.
    pub(crate) fn update_entities(&mut self, frame_time: f64) {
        // clone the pos to avoid the borrow checker's wrath
        let player_pos = CartesianPos {
            x: self.player.position.x,
            y: self.player.position.y,
        };

        let level = &mut self.config.levels[self.current_level_idx];
        let map_slice = &level.wall_map;
        let map_width = level.map_width;

        // update all entities
        for entity in &mut level.entities {
            entity.update(&player_pos, frame_time, map_slice, map_width);
        }
    }

    // changes the current level

    /// Changes the current level to next one.
    ///
    /// # Arguments
    ///
    /// * `new_level_idx` - the desired level to be changed to.
    pub fn change_level(&mut self, new_level_idx: usize) {
        // prevent crashing if player beat the last level
        if new_level_idx >= self.config.levels.len() {
            return;
        }

        self.current_level_idx = new_level_idx;

        self.textures.clear();

        for tex_path in &self.config.levels[self.current_level_idx].textures {
            self.textures.push(load_texture(tex_path));
        }

        self.player.position.x = self.config.levels[self.current_level_idx].player_start_x;
        self.player.position.y = self.config.levels[self.current_level_idx].player_start_y;
    }
}