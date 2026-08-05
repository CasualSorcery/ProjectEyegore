use crate::config::{load_config, GameConfig};
use crate::map::CartesianPos;
use crate::player::Player;
use image::{DynamicImage, RgbaImage};
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

pub struct Engine {
    config: GameConfig,
    window: Window,
    buffer: Vec<u32>,
    z_buffer: Vec<f64>,
    pub current_level_idx: usize,
    textures: Vec<Vec<u32>>,
    player: Player,
}
impl Engine {
    fn render_walls(&mut self) {
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // TODO: hardcoded variables, make them dynamic
        let tex_w: usize = 64;
        let tex_h: usize = 64;

        for x in 0..scr_w {
            // 1. Calculate ray position and direction
            let camera_x = 2.0 * (x as f64) / (scr_w as f64) - 1.0;
            let ray_dir_x = self.player.direction.x + self.player.plane.x * camera_x;
            let ray_dir_y = self.player.direction.y + self.player.plane.y * camera_x;

            let mut map_x = self.player.position.x as usize;
            let mut map_y = self.player.position.y as usize;

            let delta_dist_x = if ray_dir_x == 0.0 { 1e30 } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { 1e30 } else { (1.0 / ray_dir_y).abs() };

            let mut side_dist_x: f64;
            let mut side_dist_y: f64;
            let step_x: i32;
            let step_y: i32;

            let mut hit = false;
            let mut side = 0; // 0 for NS wall, 1 for EW wall

            // 2. Calculate step and initial sideDist
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

            // 3. Perform DDA
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

                // Use the safe helper instead of direct array access!
                if self.get_tile(map_x, map_y) > 0 {
                    hit = true;
                }
            }

            // 4. Calculate distance projected on camera direction
            let perp_wall_dist = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };

            // Save distance to Z-buffer for later sprite rendering
            self.z_buffer[x] = perp_wall_dist;

            // 5. Calculate line height and draw limits
            let line_height = (scr_h as f64 / perp_wall_dist) as isize;

            let mut draw_start = -line_height / 2 + scr_h as isize / 2;
            if draw_start < 0 { draw_start = 0; }

            let mut draw_end = line_height / 2 + scr_h as isize / 2;
            if draw_end >= scr_h as isize { draw_end = scr_h as isize - 1; }

            // 6. Texturing calculations
            // Get the tile ID. (Subtract 1 so tile ID 1 maps to texture index 0)
            let tile_id = self.get_tile(map_x, map_y);

            // Safety check: if tile_id is 0 (shouldn't happen due to 'hit'), default to 0
            let tex_num = if tile_id > 0 { (tile_id - 1) as usize } else { 0 };

            // Calculate exact hit coordinate
            let mut wall_x = if side == 0 {
                self.player.position.y + perp_wall_dist * ray_dir_y
            } else {
                self.player.position.x + perp_wall_dist * ray_dir_x
            };
            wall_x -= wall_x.floor();

            // X coordinate on the texture
            let mut tex_x = (wall_x * (tex_w as f64)) as usize;

            // Flip texture horizontally based on side
            if side == 0 && ray_dir_x > 0.0 { tex_x = tex_w - tex_x - 1; }
            if side == 1 && ray_dir_y < 0.0 { tex_x = tex_w - tex_x - 1; }

            // Texture stepping
            let step = 1.0 * (tex_h as f64) / (line_height as f64);
            let mut tex_pos = (draw_start - scr_h as isize / 2 + line_height / 2) as f64 * step;

            // 7. Draw the pixels of the vertical stripe
            for y in draw_start..=draw_end {
                let tex_y = (tex_pos as usize) & (tex_h - 1);
                tex_pos += step;

                // Safety: prevent panic if the map calls for a texture we didn't load
                let texture_slice = if tex_num < self.textures.len() {
                    &self.textures[tex_num]
                } else {
                    &self.textures[0] // Fallback to the first texture
                };

                let mut color = texture_slice[tex_h * tex_y + tex_x];

                // Fake lighting
                if side == 1 {
                    color = (color >> 1) & 0x007F7F7F;
                }

                // Write to buffer
                self.buffer[(y as usize) * scr_w + x] = color;
            }
        }
    }
    fn render_floor_ceiling(&mut self) {
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // hardcoded to 64 for now since our load_texture method forces 64x64
        // TODO: make it dynamic
        let tex_w: usize = 64;
        let tex_h: usize = 64;

        // We only loop through the bottom half of the screen (horizon down)
        for y in (scr_h / 2)..scr_h {
            // ray_dir for leftmost ray (x = 0) and rightmost ray (x = width)
            let ray_dir_x0 = self.player.direction.x - self.player.plane.x;
            let ray_dir_y0 = self.player.direction.y - self.player.plane.y;
            let ray_dir_x1 = self.player.direction.x + self.player.plane.x;
            let ray_dir_y1 = self.player.direction.y + self.player.plane.y;

            // Current y position compared to the center of the screen (the horizon)
            let p = y - (scr_h / 2);

            // Prevent division by zero at the exact center horizon line
            let p = std::cmp::max(p, 1);

            // Vertical position of the camera (half the screen height)
            let pos_z = 0.5 * scr_h as f64;

            // Horizontal distance from the camera to the floor for the current row
            let row_distance = pos_z / (p as f64);

            // Calculate the real-world step vector to add for each x
            let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / scr_w as f64;
            let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / scr_w as f64;

            // Real-world coordinates of the leftmost column
            let mut floor_x = self.player.position.x + row_distance * ray_dir_x0;
            let mut floor_y = self.player.position.y + row_distance * ray_dir_y0;

            for x in 0..scr_w {
                // The cell coord is simply the integer parts of floor_x and floor_y
                let cell_x = floor_x as i32;
                let cell_y = floor_y as i32;

                // Get the texture coordinate from the fractional part
                let tx = ((tex_w as f64 * (floor_x - cell_x as f64)) as usize) & (tex_w - 1);
                let ty = ((tex_h as f64 * (floor_y - cell_y as f64)) as usize) & (tex_h - 1);

                floor_x += floor_step_x;
                floor_y += floor_step_y;

                // TODO: hardcoded, make it dynamic
                // --- DRAW FLOOR (Using Texture 3: Greystone) ---
                let mut floor_color = self.textures[3][tex_w * ty + tx];

                // Make the floor slightly darker for a depth effect
                floor_color = (floor_color >> 1) & 0x007F7F7F;

                self.buffer[y * scr_w + x] = floor_color;

                // TODO: hardcoded, make it dynamic
                // --- DRAW CEILING (Using Texture 6: Wood) ---
                let mut ceil_color = self.textures[6][tex_w * ty + tx];
                ceil_color = (ceil_color >> 1) & 0x007F7F7F;

                // Draw symmetrically at the top of the screen
                self.buffer[(scr_h - y - 1) * scr_w + x] = ceil_color;
            }
        }
    }
    fn render_sprites(&mut self) {
        let scr_w = self.config.scr_width;
        let scr_h = self.config.scr_height;

        // TODO: hardcoded, fix later
        let tex_w: isize = 64;
        let tex_h: isize = 64;

        let level = &self.config.levels[self.current_level_idx];

        // 1. Sort sprites by distance (furthest to nearest)
        let mut sprite_order: Vec<(usize, f64)> = level.entities
            .iter()
            .enumerate()
            .map(|(i, entity)| {
                // Calculate Euclidean distance squared
                let dist = (self.player.position.x - entity.pos.x).powi(2)
                    + (self.player.position.y - entity.pos.y).powi(2);
                (i, dist)
            })
            .collect();

        // Sort descending
        sprite_order.sort_by(|a, b| b.1.total_cmp(&a.1));

        // 2. Calculate the inverse camera matrix
        let inv_det = 1.0 / (self.player.plane.x * self.player.direction.y - self.player.direction.x * self.player.plane.y);

        // 3. Draw each sprite
        for (index, _dist) in sprite_order {
            let entity = &level.entities[index];

            // Translate sprite position to relative to camera
            let sprite_x = entity.pos.x - self.player.position.x;
            let sprite_y = entity.pos.y - self.player.position.y;

            // Transform sprite with the inverse camera matrix
            let transform_x = inv_det * (self.player.direction.y * sprite_x - self.player.direction.x * sprite_y);
            let transform_y = inv_det * (-self.player.plane.y * sprite_x + self.player.plane.x * sprite_y); // Depth (Z)

            let sprite_screen_x = ((scr_w as f64 / 2.0) * (1.0 + transform_x / transform_y)) as isize;

            // Calculate height of the sprite on screen (applying the Entity's Y scale!)
            let sprite_height = ((scr_h as f64 / transform_y.abs()) * entity.scale_y) as isize;

            let mut draw_start_y = -sprite_height / 2 + scr_h as isize / 2;
            if draw_start_y < 0 { draw_start_y = 0; }
            let mut draw_end_y = sprite_height / 2 + scr_h as isize / 2;
            if draw_end_y >= scr_h as isize { draw_end_y = scr_h as isize - 1; }

            // Calculate width of the sprite (applying the Entity's X scale!)
            let sprite_width = ((scr_h as f64 / transform_y.abs()) * entity.scale_x) as isize;

            let mut draw_start_x = -sprite_width / 2 + sprite_screen_x;
            if draw_start_x < 0 { draw_start_x = 0; }
            let mut draw_end_x = sprite_width / 2 + sprite_screen_x;
            if draw_end_x >= scr_w as isize { draw_end_x = scr_w as isize - 1; }

            // 4. Render the vertical stripes
            for stripe in draw_start_x..draw_end_x {
                let mut tex_x = (256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * tex_w / sprite_width) / 256;

                // Clamp tex_x
                if tex_x < 0 { tex_x = 0; }
                if tex_x >= tex_w { tex_x = tex_w - 1; }

                let stripe_usize = stripe as usize;

                // --- Z-BUFFER CHECK ---
                if transform_y > 0.0 && stripe > 0 && stripe < scr_w as isize && transform_y < self.z_buffer[stripe_usize] {
                    for y in draw_start_y..draw_end_y {
                        let d = y * 256 - scr_h as isize * 128 + sprite_height * 128;
                        let mut tex_y = ((d * tex_h) / sprite_height) / 256;

                        // Clamp tex_y
                        if tex_y < 0 { tex_y = 0; }
                        if tex_y >= tex_h { tex_y = tex_h - 1; }

                        // Safety check: ensure the texture exists
                        let tex_idx = if entity.texture < self.textures.len() { entity.texture } else { 0 };
                        let color = self.textures[tex_idx][(tex_h as usize) * (tex_y as usize) + (tex_x as usize)];

                        // Mask out pure black pixels (transparency)
                        if (color & 0x00FFFFFF) != 0 {
                            self.buffer[(y as usize) * scr_w + stripe_usize] = color;
                        }
                    }
                }
            }
        }
    }
    fn get_tile(&self, x: usize, y: usize) -> u8 {
        let level = &self.config.levels[self.current_level_idx];

        let index = x * level.map_width + y;

        *level.map.get(index).unwrap_or(&1)
    }
    pub fn change_level(&mut self, new_level_idx: usize) {
        // Prevent crashing if they beat the last level
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
    fn handle_input(&mut self, frame_time: f64) {
        let move_step = frame_time * self.player.move_speed;
        let rotation_step = frame_time * self.player.rotation_speed;

        // move forward
        if self.window.is_key_down(Key::Up) || self.window.is_key_down(Key::W) {
            let next_x = self.player.position.x + self.player.direction.x * move_step;
            let next_y = self.player.position.y + self.player.direction.y * move_step;

            // check X collision
            if self.get_tile(next_x as usize, self.player.position.y as usize) == 0 {
                self.player.position.x = next_x;
            }
            // check Y collision
            if self.get_tile(self.player.position.x as usize, next_y as usize) == 0 {
                self.player.position.y = next_y;
            }
        }

        // move backward
        if self.window.is_key_down(Key::Down) || self.window.is_key_down(Key::S) {
            let next_x = self.player.position.x - self.player.direction.x * move_step;
            let next_y = self.player.position.y - self.player.direction.y * move_step;

            // check X collision
            if self.get_tile(next_x as usize, self.player.position.y as usize) == 0 {
                self.player.position.x = next_x;
            }
            // check Y collision
            if self.get_tile(self.player.position.x as usize, next_y as usize) == 0 {
                self.player.position.y = next_y;
            }
        }

        // rotate right
        if self.window.is_key_down(Key::Right) || self.window.is_key_down(Key::D) {
            let cos_rot = (-rotation_step).cos();
            let sin_rot = (-rotation_step).sin();

            let old_dir_x = self.player.direction.x;
            self.player.direction.x = self.player.direction.x * cos_rot - self.player.direction.y * sin_rot;
            self.player.direction.y = old_dir_x * sin_rot + self.player.direction.y * cos_rot;

            let old_plane_x = self.player.plane.x;
            self.player.plane.x = self.player.plane.x * cos_rot - self.player.plane.y * sin_rot;
            self.player.plane.y = old_plane_x * sin_rot + self.player.plane.y * cos_rot;
        }

        // rotate left
        if self.window.is_key_down(Key::Left) || self.window.is_key_down(Key::A) {
            let cos_rot = rotation_step.cos();
            let sin_rot = rotation_step.sin();

            let old_dir_x = self.player.direction.x;
            self.player.direction.x = self.player.direction.x * cos_rot - self.player.direction.y * sin_rot;
            self.player.direction.y = old_dir_x * sin_rot + self.player.direction.y * cos_rot;

            let old_plane_x = self.player.plane.x;
            self.player.plane.x = self.player.plane.x * cos_rot - self.player.plane.y * sin_rot;
            self.player.plane.y = old_plane_x * sin_rot + self.player.plane.y * cos_rot;
        }
    }
    fn update_entities(&mut self, frame_time: f64) {
        let player_pos = CartesianPos {
            x: self.player.position.x,
            y: self.player.position.y,
        };

        let level = &mut self.config.levels[self.current_level_idx];

        let map_slice = &level.map;
        let map_width = level.map_width;

        for entity in &mut level.entities {
            entity.update(&player_pos, frame_time, map_slice, map_width);
        }
    }
    pub fn new(filepath: &str) -> Self {
        let config = load_config(filepath);

        let window = create_window(&config, WindowOptions::default());

        if config.levels.is_empty() {
            panic!("Configuration file has no levels!");
        }

        let mut textures: Vec<Vec<u32>> = Vec::with_capacity(config.levels[0].textures.len());

        for tx in &config.levels[0].textures {
            textures.push(load_texture(tx));
        }

        let buffer_size = config.scr_width * config.scr_height;

        let z_buffer_size = config.scr_width;

        let starting_player = Player {
            position: CartesianPos {
                x: config.levels[0].player_start_x,
                y: config.levels[0].player_start_y,
            },
            direction: CartesianPos { x: -1.0, y: 0.0 },
            plane: CartesianPos { x: 0.0, y: 0.66 },
            move_speed: 5.0,
            rotation_speed: 3.0,
        };

        Self {
            config,
            window,
            buffer: vec![0; buffer_size],
            z_buffer: vec![0.0; z_buffer_size],
            textures,
            current_level_idx: 0,
            player: starting_player,
        }
    }
    pub fn run(&mut self) {
        let mut current_time = Instant::now();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            // calculating delta time
            let new_time = Instant::now();
            let frame_time = new_time.duration_since(current_time).as_secs_f64();
            current_time = new_time;

            // input handling
            self.handle_input(frame_time);

            self.update_entities(frame_time);

            self.render_floor_ceiling();

            self.render_walls();

            self.render_sprites();

            // update window
            self.window.update_with_buffer(
                &self.buffer,
                self.config.scr_width,
                self.config.scr_height)
                .unwrap();
        }
    }
}

// loads/parses a single 64x64 png texture, returns the texture u32 vector
fn load_texture(filepath: &str) -> Vec<u32> {
    // tries to open the image file (png only), panics if can't
    let img: DynamicImage = image::open(filepath).unwrap_or_else(|e| {
        panic!("Failed to open image {}: {}", filepath, e)
    });

    // ensures 64x64 pixel size
    let img: DynamicImage = img.resize_exact(
        64, 64,
        image::imageops::FilterType::Nearest,
    );

    // transforms the images to rgba format
    let rgba_image: RgbaImage = img.to_rgba8();

    // initializes the buffer where the textures will be held
    let mut texture_buffer: Vec<u32> = vec![0; 64 * 64];

    // loops through each pixel, unpacks them to the rgb value, and appends each to the texture buffer
    for (x, y, pixel) in rgba_image.enumerate_pixels() {
        let r: u32 = pixel[0] as u32;
        let g: u32 = pixel[1] as u32;
        let b: u32 = pixel[2] as u32;

        // minifb uses 0x00RRGGBB (ignoring alpha)
        let color: u32 = (r << 16) + (g << 8) + b;
        texture_buffer[(y as usize) * 64 + (x as usize)] = color;
    }
    // returns the image as an u32 vector of RGB pixels
    texture_buffer
}

// create a window using the params specifications
// since Rust doesn't support optional args natively, you must call this function implementing ALL Params
fn create_window(options: &GameConfig, window_options: WindowOptions) -> Window {
    // creates a window object using minifb crate
    let mut window: Window = Window::new(
        &options.name, // window name only accepts &str
        options.scr_width,
        options.scr_height,
        window_options,
    ).unwrap_or_else(|e| {  // "work or panic" basically
        panic!("Failed to create window: {}", e);
    });

    // sets the fps limit & target for application
    window.set_target_fps(options.target_fps);

    // returns the window object
    window
}