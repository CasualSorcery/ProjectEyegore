use super::Engine;
use crate::utils::helpers::load_texture;
use crate::world::map::CartesianPos;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode};
use std::time::Instant;

impl Engine {
    // helper function, gets the tile according to the coordinates

    /// Gets a single, specific tile according to the provided coordinates in the current level.
    ///
    /// # Arguments
    ///
    /// * `x` - the x coordinate of the tile.
    /// * `y` - the y coordinate of the tile.
    ///
    /// # Returns
    ///
    /// * the `u8` address of the tile.
    #[inline(always)]
    pub(crate) fn get_tile(&self, x: usize, y: usize) -> u8 {
        let level = &self.config.levels[self.current_level_idx];

        let index = x * level.map_width + y;

        *level.map.get(index).unwrap_or(&1)
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

        // pause screen handle
        if self.window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            self.is_paused = !self.is_paused;

            // reset mouse pos
            if !self.is_paused {
                self.input.last_mouse_x = None;
            }
        }

        // if game is paused release the mouse control
        if self.is_paused {
            // We MUST ensure the mouse is visible and free to leave the window when paused!
            self.window.set_cursor_visibility(true);
            return;
        }

        // else cursor is to remain not visible
        self.window.set_cursor_visibility(false);

        // mouse configs
        let mouse_sensitivity = 0.003;

        // ------------------------------------------------------------
        // movement and vision control
        // ------------------------------------------------------------

        let move_step = frame_time * self.player.move_speed;

        let right_dir_x = -self.player.direction.y;
        let right_dir_y = self.player.direction.x;

        let mut input_x = 0.0;
        let mut input_y = 0.0;

        // normalized spatial movement input
        // TODO: make controls dynamic (ex.: w or up, s or down, a or right ...)
        if self.window.is_key_down(Key::W) {
            input_y += 1.0;
        }
        if self.window.is_key_down(Key::S) {
            input_y -= 1.0;
        }
        if self.window.is_key_down(Key::D) {
            input_x -= 1.0;
        }
        if self.window.is_key_down(Key::A) {
            input_x += 1.0;
        }

        if input_x != 0.0 || input_y != 0.0 {
            // if moving diagonally, normalize speed
            if input_x != 0.0 && input_y != 0.0 {
                let inv_sqrt2 = std::f64::consts::FRAC_1_SQRT_2; // famous fast inverse sqr root
                input_x *= inv_sqrt2;
                input_y *= inv_sqrt2;
            }

            // calculate final movement vectors
            let move_vec_x =
                (self.player.direction.x * input_y + right_dir_x * input_x) * move_step;
            let move_vec_y =
                (self.player.direction.y * input_y + right_dir_y * input_x) * move_step;

            let next_x = self.player.position.x + move_vec_x;
            let next_y = self.player.position.y + move_vec_y;

            // apply collision
            if self.get_tile(next_x as usize, self.player.position.y as usize) == 0 {
                self.player.position.x = next_x;
            }
            if self.get_tile(self.player.position.x as usize, next_y as usize) == 0 {
                self.player.position.y = next_y;
            }
        }

        // horizontal camera movement
        if let Some((mouse_x, _mouse_y)) = self.window.get_mouse_pos(MouseMode::Pass) {
            // normal rotation math
            if let Some(last_x) = self.input.last_mouse_x {
                let mouse_delta_x = (mouse_x - last_x) as f64;

                if mouse_delta_x != 0.0 && mouse_delta_x.abs() < 100.0 {
                    let rot_step = -mouse_delta_x * mouse_sensitivity;
                    let cos_rot = rot_step.cos();
                    let sin_rot = rot_step.sin();

                    let old_dir_x = self.player.direction.x;
                    self.player.direction.x =
                        self.player.direction.x * cos_rot - self.player.direction.y * sin_rot;
                    self.player.direction.y =
                        old_dir_x * sin_rot + self.player.direction.y * cos_rot;

                    let old_plane_x = self.player.plane.x;
                    self.player.plane.x =
                        self.player.plane.x * cos_rot - self.player.plane.y * sin_rot;
                    self.player.plane.y = old_plane_x * sin_rot + self.player.plane.y * cos_rot;
                }
            }

            // edge warping check
            // if mouse out of the window, snap back to center of the game screen
            if mouse_x < 50.0 || mouse_x > (self.config.scr_width as f32 - 50.0) {
                let (win_x, win_y) = self.window.get_position();

                // get the physical pixels of the window on the monitor
                let (win_w, win_h) = self.window.get_size();

                let monitor_center_x = win_x as i32 + (win_w as i32 / 2);
                let monitor_center_y = win_y as i32 + (win_h as i32 / 2);

                #[cfg(windows)]
                unsafe {
                    windows_sys::Win32::UI::WindowsAndMessaging::SetCursorPos(
                        monitor_center_x,
                        monitor_center_y,
                    );
                }
                self.input.last_mouse_x = None;
            } else {
                self.input.last_mouse_x = Some(mouse_x);
            }
        } else {
            // else just reset mouse pos
            self.input.last_mouse_x = None;
        }

        // ------------------------------------------------------------
        // interaction controls
        // ------------------------------------------------------------

        // interact
        if self.window.is_key_pressed(Key::E, KeyRepeat::No) {
            self.player.inventory.change_weapon();
        }

        // shoot
        if self.window.get_mouse_down(MouseButton::Left) {
            let now = Instant::now();

            // fire rate
            if now.duration_since(self.input.last_shot_time).as_secs_f64() > 0.3 {
                let mut damage_to_deal = 0.0;

                // check weapon and consume ammo
                if let Some(crate::world::player::Items::Weapon {
                    damage,
                    ammo,
                    name: _,
                }) = self.player.inventory.get_current_wpn_mut()
                {
                    if *ammo > 0 {
                        *ammo -= 1;
                        damage_to_deal = *damage;
                        self.input.last_shot_time = now; // Reset cooldown
                    } else {
                        // no ammo
                    }
                }

                // 2. If we actually fired a shot, do the hitscan math!
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

                            // If enemy is in crosshairs (< 0.2 radians) and is closest
                            if angle_diff.abs() < 0.2 && dist < closest_dist {
                                closest_dist = dist;
                                hit_index = Some(i);
                            }
                        }
                    }

                    // 3. Apply Damage to the specific enemy
                    if let Some(idx) = hit_index
                        && let crate::world::entity::EntityType::Enemy { ref mut hp, .. } =
                            level.entities[idx].entity_type
                    {
                        *hp -= damage_to_deal;
                    }
                }
            }
        }

        // ------------------------------------------------------------
        // debug
        // ------------------------------------------------------------

        // toggle debug mode
        if self.window.is_key_pressed(Key::F3, KeyRepeat::No) {
            self.show_debug = !self.show_debug;
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

        let map_slice = &level.map;
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

        for tx_path in &self.config.levels[self.current_level_idx].textures {
            self.textures.push(load_texture(tx_path));
        }

        self.player.position.x = self.config.levels[self.current_level_idx].player_start_x;
        self.player.position.y = self.config.levels[self.current_level_idx].player_start_y;
    }
}
