use super::Engine;
use font8x8::UnicodeFonts;
use std::fmt::Write;

// constant size of textures
const TEX_SIZE: usize = 64;

impl Engine {
    // -------------------------------------------------------
    // main rendering functions
    // -------------------------------------------------------

    /// Renders the walls by making ray-casting calculations,
    /// writes the result in the Engine's `buffer` and `z_buffer`.
    pub(crate) fn render_walls(&mut self) {
        // screen size to render
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // fog distance
        let fog_dist = self.config.levels[self.current_level_idx].max_fog_distance;

        // main loop of wall ray-casting
        for x in 0..scr_w {
            // step 1 - calculate ray position and direction
            let camera_x = 2.0 * (x as f64) / (scr_w as f64) - 1.0;
            let ray_dir_x = self.player.direction.x + self.player.plane.x * camera_x;
            let ray_dir_y = self.player.direction.y + self.player.plane.y * camera_x;

            let mut map_x = self.player.position.x as usize;
            let mut map_y = self.player.position.y as usize;

            let delta_dist_x: f64 = if ray_dir_x == 0.0 {
                1e30
            } else {
                (1.0 / ray_dir_x).abs()
            };
            let delta_dist_y: f64 = if ray_dir_y == 0.0 {
                1e30
            } else {
                (1.0 / ray_dir_y).abs()
            };

            let mut side_dist_x: f64;
            let mut side_dist_y: f64;

            let step_x: i32;
            let step_y: i32;

            let mut hit = false;
            let mut side = 0; // 0 for NS wall, 1 for EW wall

            // step 2 - calculate step and initial side_dist
            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (self.player.position.x - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - self.player.position.x) * delta_dist_x;
            }

            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (self.player.position.y - map_y as f64) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - self.player.position.y) * delta_dist_y;
            }

            // step 3 - perform DDA
            while !hit {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x = (map_x as i32 + step_x) as usize;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y = (map_y as i32 + step_y) as usize;
                    side = 1;
                }

                // check if hit wall
                if self.get_tile(map_x, map_y) > 0 {
                    hit = true;
                }
            }

            // step 4 - calculate distance projected on camera direction
            let perp_wall_dist = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };

            // save distance to Z-buffer for later sprite rendering
            self.z_buffer[x] = perp_wall_dist;

            // step 5 - calculate line height and draw limits
            let line_height = (scr_h as f64 / perp_wall_dist) as isize;

            let mut draw_start = -line_height / 2 + scr_h as isize / 2;
            if draw_start < 0 {
                draw_start = 0;
            }

            let mut draw_end = line_height / 2 + scr_h as isize / 2;
            if draw_end >= scr_h as isize {
                draw_end = scr_h as isize - 1;
            }

            // step 6 - texturing calculations
            let tile_id = self.get_tile(map_x, map_y);

            // safety check - if tile_id is 0, default to 0
            let tex_num = if tile_id > 0 {
                (tile_id - 1) as usize
            } else {
                0
            };

            // calculate exact hit coordinate
            let mut wall_x = if side == 0 {
                self.player.position.y + perp_wall_dist * ray_dir_y
            } else {
                self.player.position.x + perp_wall_dist * ray_dir_x
            };
            wall_x -= wall_x.floor();

            // X coordinate on the texture
            let mut tex_x = (wall_x * (TEX_SIZE as f64)) as usize;

            // flip texture horizontally based on side
            if side == 0 && ray_dir_x > 0.0 {
                tex_x = TEX_SIZE - tex_x - 1;
            }
            if side == 1 && ray_dir_y < 0.0 {
                tex_x = TEX_SIZE - tex_x - 1;
            }

            // texture stepping
            let step = 1.0 * (TEX_SIZE as f64) / (line_height as f64);
            let mut tex_pos = (draw_start - scr_h as isize / 2 + line_height / 2) as f64 * step;

            // step 7 - draw the pixels of the vertical stripe
            for y in draw_start..=draw_end {
                let tex_y = (tex_pos as usize) & (TEX_SIZE - 1);
                tex_pos += step;

                // safety: prevent panic if the map calls for a texture that didn't load
                let texture_slice = if tex_num < self.textures.len() {
                    &self.textures[tex_num]
                } else {
                    &self.textures[0] // fallback to first texture
                };

                let mut color = texture_slice[TEX_SIZE * tex_y + tex_x];

                // fake lighting
                if side == 1 {
                    color = (color >> 1) & 0x007F7F7F;
                }

                // fog effect
                color = Engine::shade_color(color, perp_wall_dist, fog_dist);

                // write to buffer
                self.buffer[(y as usize) * scr_w + x] = color;
            }
        }
    }

    /// Renders the floor and ceiling by making ray-casting calculations,
    /// writes the result in the Engine's `buffer` and `z_buffer`.
    pub(crate) fn render_floor_ceiling(&mut self) {
        // screen size to render
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // grabbing the level config values
        let level = &self.config.levels[self.current_level_idx];
        let floor_idx = level.floor_tex_idx;
        let ceil_idx = level.ceil_tex_idx;

        // fog distance
        let fog_dist = self.config.levels[self.current_level_idx].max_fog_distance;

        // loop only through the bottom half of the screen
        for y in (scr_h / 2)..scr_h {
            // ray_dir for leftmost ray (x = 0) and rightmost ray (x = width)
            let ray_dir_x0 = self.player.direction.x - self.player.plane.x;
            let ray_dir_y0 = self.player.direction.y - self.player.plane.y;
            let ray_dir_x1 = self.player.direction.x + self.player.plane.x;
            let ray_dir_y1 = self.player.direction.y + self.player.plane.y;

            // current y position compared to the center of the screen (the horizon)
            let p = y - (scr_h / 2);

            // prevent division by zero at the exact center horizon line
            let p = std::cmp::max(p, 1);

            // vertical position of the camera (half the screen height)
            let pos_z = 0.5 * scr_h as f64;

            // horizontal distance from the camera to the floor for the current row
            let row_distance = pos_z / (p as f64);

            // calculate the real-world step vector to add for each x
            let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / scr_w as f64;
            let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / scr_w as f64;

            // real-world coordinates of leftmost column
            let mut floor_x = self.player.position.x + row_distance * ray_dir_x0;
            let mut floor_y = self.player.position.y + row_distance * ray_dir_y0;

            for x in 0..scr_w {
                // the cell coord is the integer parts of floor_x and floor_y
                let cell_x = floor_x as i32;
                let cell_y = floor_y as i32;

                // get the texture coordinate from the fractional part
                let tx = ((TEX_SIZE as f64 * (floor_x - cell_x as f64)) as usize) & (TEX_SIZE - 1);
                let ty = ((TEX_SIZE as f64 * (floor_y - cell_y as f64)) as usize) & (TEX_SIZE - 1);

                floor_x += floor_step_x;
                floor_y += floor_step_y;

                // draw floor
                let mut floor_color = self.textures[floor_idx][TEX_SIZE * ty + tx];

                // fake lightning
                floor_color = (floor_color >> 1) & 0x007F7F7F;

                // fog effect
                floor_color = Engine::shade_color(floor_color, row_distance, fog_dist);

                self.buffer[y * scr_w + x] = floor_color;

                // draw ceiling
                let mut ceil_color = self.textures[ceil_idx][TEX_SIZE * ty + tx];
                ceil_color = (ceil_color >> 1) & 0x007F7F7F;

                // fog effect
                ceil_color = Engine::shade_color(ceil_color, row_distance, fog_dist);

                // draw symmetrically at the top of the screen
                self.buffer[(scr_h - y - 1) * scr_w + x] = ceil_color;
            }
        }
    }

    /// Renders the sprites by making ray-casting calculations,
    /// writes the result in the Engine's `buffer`, `z_buffer` and `sprites_buffer`.
    pub(crate) fn render_sprites(&mut self) {
        // screen size to render
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // fog distance
        let fog_dist = self.config.levels[self.current_level_idx].max_fog_distance;

        let level = &self.config.levels[self.current_level_idx];

        self.sprite_buffer.clear();

        // step 1 - Sort sprites by distance (furthest to nearest)
        for (i, entity) in level.entities.iter().enumerate() {
            let dist = (self.player.position.x - entity.pos.x).powi(2)
                + (self.player.position.y - entity.pos.y).powi(2);
            self.sprite_buffer.push((i, dist));
        }

        // sort descending
        self.sprite_buffer
            .sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

        // step 2 - calculate the inverse camera matrix
        let inv_det = 1.0
            / (self.player.plane.x * self.player.direction.y
                - self.player.direction.x * self.player.plane.y);

        // step 3 - draw each sprite
        for &(index, _dist) in &self.sprite_buffer {
            let entity = &level.entities[index];

            // translate sprite position to relative to camera
            let sprite_x = entity.pos.x - self.player.position.x;
            let sprite_y = entity.pos.y - self.player.position.y;

            // transform sprite with the inverse camera matrix
            let transform_x =
                inv_det * (self.player.direction.y * sprite_x - self.player.direction.x * sprite_y);
            let transform_y =
                inv_det * (-self.player.plane.y * sprite_x + self.player.plane.x * sprite_y); // Depth (Z)

            let sprite_screen_x =
                ((scr_w as f64 / 2.0) * (1.0 + transform_x / transform_y)) as isize;

            // calculate height of the sprite
            let sprite_height = ((scr_h as f64 / transform_y.abs()) * entity.scale_y) as isize;

            let mut draw_start_y = -sprite_height / 2 + scr_h as isize / 2;
            if draw_start_y < 0 {
                draw_start_y = 0;
            }
            let mut draw_end_y = sprite_height / 2 + scr_h as isize / 2;
            if draw_end_y >= scr_h as isize {
                draw_end_y = scr_h as isize - 1;
            }

            // calculate width of the sprite
            let sprite_width = ((scr_h as f64 / transform_y.abs()) * entity.scale_x) as isize;

            let mut draw_start_x = -sprite_width / 2 + sprite_screen_x;
            if draw_start_x < 0 {
                draw_start_x = 0;
            }
            let mut draw_end_x = sprite_width / 2 + sprite_screen_x;
            if draw_end_x >= scr_w as isize {
                draw_end_x = scr_w as isize - 1;
            }

            // step 4 - render the vertical stripes
            for stripe in draw_start_x..draw_end_x {
                let mut tex_x =
                    (256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * TEX_SIZE as isize
                        / sprite_width)
                        / 256;

                // clamp tex_x
                if tex_x < 0 {
                    tex_x = 0;
                }
                if tex_x >= TEX_SIZE as isize {
                    tex_x = TEX_SIZE as isize - 1;
                }

                let stripe_usize = stripe as usize;

                // Z-BUFFER CHECK
                if transform_y > 0.0
                    && stripe > 0
                    && stripe < scr_w as isize
                    && transform_y < self.z_buffer[stripe_usize]
                {
                    for y in draw_start_y..draw_end_y {
                        let d = y * 256 - scr_h as isize * 128 + sprite_height * 128;
                        let mut tex_y = ((d * TEX_SIZE as isize) / sprite_height) / 256;

                        // clamp tex_y
                        if tex_y < 0 {
                            tex_y = 0;
                        }
                        if tex_y >= TEX_SIZE as isize {
                            tex_y = TEX_SIZE as isize - 1;
                        }

                        // safety check: ensure the texture exists
                        let tex_idx = if entity.texture < self.textures.len() {
                            entity.texture
                        } else {
                            0
                        };
                        let mut color = self.textures[tex_idx]
                            [(TEX_SIZE) * (tex_y as usize) + (tex_x as usize)];

                        // mask out pure black pixels (transparency)
                        if (color & 0x00FFFFFF) != 0 {
                            // fog effect
                            color = Engine::shade_color(color, transform_y, fog_dist);

                            self.buffer[(y as usize) * scr_w + stripe_usize] = color;
                        }
                    }
                }
            }
        }
    }

    // helper function,
    // this says here because it is frequently called,
    // and it needs to be made easy access.

    /// Writes text to the as an overlay.
    ///
    /// updates the buffer provide to contain the provided text drawn upon it.
    ///
    /// # Arguments
    ///
    /// * `buffer` - the u32 Vector buffer in which the text will be drawn upon.
    /// * `scr_w` - the width of the screen to be drawn on.
    /// * `scr_h` - the height of the screen to be drawn on.
    /// * `text` - the text to be drawn
    /// * `start_x` - on-screen x coordinates where the text should start.
    /// * `start_y` - on-screen x coordinates where the text should start.
    /// * `scale` - scale of the drawn text
    #[inline(always)]
    pub(crate) fn draw_text(
        buffer: &mut [u32],
        scr_w: usize,
        scr_h: usize,
        text: &str,
        start_x: usize,
        start_y: usize,
        scale: usize,
    ) {
        let mut current_x = start_x;

        // iterates over every single char in the text input
        for ch in text.chars() {
            if let Some(bitmap) = font8x8::BASIC_FONTS.get(ch) {
                for (row_idx, row_byte) in bitmap.iter().enumerate() {
                    for bit_idx in 0..8 {
                        if (*row_byte >> bit_idx) & 1 == 1 {
                            for sy in 0..scale {
                                for sx in 0..scale {
                                    let pixel_x = current_x + (bit_idx * scale) + sx;
                                    let pixel_y = start_y + (row_idx * scale) + sy;

                                    // check bounds against the passed scr width and scr height
                                    if pixel_x < scr_w && pixel_y < scr_h {
                                        let index = pixel_y * scr_w + pixel_x;
                                        buffer[index] = 0x00FFFFFF;
                                    }
                                }
                            }
                        }
                    }
                }
                current_x += 8 * scale + scale;
            }
        }
    }

    // apply shading to pixels

    /// Shades colored pixels based on intensity
    ///
    /// # Arguments
    ///
    /// * `color` - The u32 color pixel to be shaded.
    /// * `distance` - The base distance between the camera and the pixel.
    /// * `max_distance` - The max distance between the camera and the pixel.
    ///
    /// # Returns
    ///
    /// * the already shaded u32 pixel.
    #[inline(always)]
    pub(crate) fn shade_color(color: u32, distance: f64, max_distance: f64) -> u32 {
        // light intensity, 1.0 = full bright, 0.0 = pitch black
        let mut intensity = 1.0 - (distance / max_distance);

        // clamp intensity
        intensity = intensity.clamp(0.0, 1.0);

        // extracting rgb values
        let r = ((color >> 16) & 0xFF) as f64;
        let g = ((color >> 8) & 0xFF) as f64;
        let b = (color & 0xFF) as f64;

        // apply the intensity
        let shade_r = (r * intensity) as u32;
        let shade_g = (g * intensity) as u32;
        let shade_b = (b * intensity) as u32;

        // repack into an u32 rgb pixel
        (shade_r << 16) | (shade_g << 8) | shade_b
    }

    // a custom debug render to show current fps and player pos

    /// Renders a custom debug overlay containing:
    ///
    /// * Fps meter
    /// * Player position
    /// * Crosshair
    ///
    /// # Arguments
    ///
    /// * `frame_time` - Delta time in seconds since last frame.
    pub(crate) fn render_debug_overlay(&mut self, frame_time: f64) {
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // clear debug string buffer
        self.debug_string.clear();

        // fps meter
        let _ = write!(&mut self.debug_string, "FPS: {:0}", 1.0 / frame_time);

        // draw the fps meter
        Engine::draw_text(
            &mut self.buffer,
            scr_w,
            scr_h,
            &self.debug_string,
            10,
            10,
            2,
        );

        // clear debug string buffer
        self.debug_string.clear();

        // player position
        let _ = write!(
            &mut self.debug_string,
            "POS: X:{:.1} Y:{:.1}",
            self.player.position.x, self.player.position.y
        );

        // draw the player position
        Engine::draw_text(
            &mut self.buffer,
            scr_w,
            scr_h,
            &self.debug_string,
            10,
            30,
            2,
        );

        // clear debug string buffer
        self.debug_string.clear();

        // player status
        let _ = write!(
            &mut self.debug_string,
            "HP: {:.0} ARMOR: {:.0}",
            self.player.health, self.player.armor
        );

        // draw player status
        Engine::draw_text(
            &mut self.buffer,
            scr_w,
            scr_h,
            &self.debug_string,
            10,
            50,
            2,
        );

        // get the screen center
        let center_x = scr_w / 2;
        let center_y = scr_h / 2;

        // draw crosshair
        Engine::draw_text(
            &mut self.buffer,
            scr_w,
            scr_h,
            "+",
            center_x - 8,
            center_y - 8,
            2,
        );
    }
}
