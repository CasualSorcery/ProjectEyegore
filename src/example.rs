use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

// constants
const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 480;
const TEX_WIDTH: usize = 64;
const TEX_HEIGHT: usize = 64;
const MAP_WIDTH: usize = 24;
const MAP_HEIGHT: usize = 24;

// structs
struct Sprite {
    x: f64,
    y: f64,
    texture: usize,
}

// world map 2d array
static WORLD_MAP: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,7,7,7,7,7,7,7,7],
    [4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,7],
    [4,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7],
    [4,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7],
    [4,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,7],
    [4,0,4,0,0,0,0,5,5,5,5,5,5,5,5,5,7,7,0,7,7,7,7,7],
    [4,0,5,0,0,0,0,5,0,5,0,5,0,5,0,5,7,0,0,0,7,7,7,1],
    [4,0,6,0,0,0,0,5,0,0,0,0,0,0,0,5,7,0,0,0,0,0,0,8],
    [4,0,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,7,7,1],
    [4,0,8,0,0,0,0,5,0,0,0,0,0,0,0,5,7,0,0,0,0,0,0,8],
    [4,0,0,0,0,0,0,5,0,0,0,0,0,0,0,5,7,0,0,0,7,7,7,1],
    [4,0,0,0,0,0,0,5,5,5,5,0,5,5,5,5,7,7,7,7,7,7,7,1],
    [6,6,6,6,6,6,6,6,6,6,6,0,6,6,6,6,6,6,6,6,6,6,6,6],
    [8,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4],
    [6,6,6,6,6,6,0,6,6,6,6,0,6,6,6,6,6,6,6,6,6,6,6,6],
    [4,4,4,4,4,4,0,4,4,4,6,0,6,2,2,2,2,2,2,2,3,3,3,3],
    [4,0,0,0,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,0,0,0,2],
    [4,0,0,0,0,0,0,0,0,0,0,0,6,2,0,0,5,0,0,2,0,0,0,2],
    [4,0,0,0,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,2,0,2,2],
    [4,0,6,0,6,0,0,0,0,4,6,0,0,0,0,0,5,0,0,0,0,0,0,2],
    [4,0,0,5,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,2,0,2,2],
    [4,0,6,0,6,0,0,0,0,4,6,0,6,2,0,0,5,0,0,2,0,0,0,2],
    [4,0,0,0,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,0,0,0,2],
    [4,4,4,4,4,4,4,4,4,4,1,1,1,2,2,2,2,2,2,3,3,3,3,3]
];

// helper functions
fn load_texture(filepath: &str) -> Vec<u32>{
    // Open the image file
    let img: image::DynamicImage = image::open(filepath).unwrap_or_else(|e| {
        panic!("Failed to open image {}: {}", filepath, e);
    });

    // Ensure it matches 64x64 expectation
    let img: image::DynamicImage = img.resize_exact(
        64, 64,
        image::imageops::FilterType::Nearest
    );

    let rgba_image: image::RgbaImage = img.to_rgba8();
    let mut texture_buffer: Vec<u32> = vec![0; 64 * 64];

    // Convert [R, G, B, A] bytes into a single u32 ARGB number
    for (x, y, pixel) in rgba_image.enumerate_pixels() {
        let r: u32 = pixel[0] as u32;
        let g: u32 = pixel[1] as u32;
        let b: u32 = pixel[2] as u32;
        // let a = pixel[3] as u32; // Assuming fully opaque for walls

        // minifb uses 0x00RRGGBB (ignoring alpha for now)
        let color: u32 = (r << 16) | (g << 8) | b;

        texture_buffer[(y as usize) * 64 + (x as usize)] = color;
    }

    texture_buffer
}

// main functions
fn game_window() {
    // the pixel screen buffer
    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let mut z_buffer: Vec<f64> = vec![0.0; SCREEN_WIDTH];

    // window object
    let mut window: Window = Window::new(
        "EyeGore", // Name of the window
        SCREEN_WIDTH, // width in pixels of the window
        SCREEN_HEIGHT, // height in pixels of the window
        WindowOptions::default(), // default misc options
    ).unwrap_or_else(|e| { // "either work or throw a panic"
        panic!("Failed to create window: {}", e);
    });

    (&mut window).set_target_fps(60); // setting the universal truth

    // current frame
    let mut current_time: Instant = Instant::now();

    // X and Y of the starting position
    let mut pos_x: f64 = 22.00;
    let mut pos_y: f64 = 11.50;

    // initial direction vector
    let mut dir_x: f64 = -1.00;
    let mut dir_y: f64 = 0.00;

    // 2d raycaster version of the camera plane
    let mut plane_x: f64 = 0.00;
    let mut plane_y: f64 = 0.66;

    // 2d vector that allocates at least 10 secured texture slots
    let mut textures: Vec<Vec<u32>> = Vec::with_capacity(10);

    // allocates the sprites into the coords
    let sprites: Vec<Sprite> = vec![
        Sprite { x: 20.5, y: 11.5, texture: 8 }, // Green barrel
        Sprite { x: 18.5, y: 4.5, texture: 9 },  // Pillar
        Sprite { x: 10.0, y: 4.5, texture: 8 }, // Green barrel
    ];

    // load the textures into memory
    (&mut textures).push(load_texture("assets/pics/eagle.png"));      // Texture 0
    (&mut textures).push(load_texture("assets/pics/redbrick.png"));   // Texture 1
    (&mut textures).push(load_texture("assets/pics/purplestone.png"));// Texture 2
    (&mut textures).push(load_texture("assets/pics/greystone.png"));  // Texture 3
    (&mut textures).push(load_texture("assets/pics/bluestone.png"));  // Texture 4
    (&mut textures).push(load_texture("assets/pics/mossy.png"));      // Texture 5
    (&mut textures).push(load_texture("assets/pics/wood.png"));       // Texture 6
    (&mut textures).push(load_texture("assets/pics/colorstone.png")); // Texture 7
    (&mut textures).push(load_texture("assets/pics/barrel.png"));     // Texture 8
    (&mut textures).push(load_texture("assets/pics/pillar.png"));     // Texture 9

    // window main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Timing
        let new_time: Instant = Instant::now();
        let frame_time: f64 = new_time.duration_since(current_time).as_secs_f64();
        current_time = new_time;

        // movement and rotation
        let move_speed: f64 = frame_time * 5.0;
        let rot_speed: f64 = frame_time * 3.0;

        // floor and ceiling ray-casting
        for y in (SCREEN_HEIGHT / 2)..SCREEN_HEIGHT {
            // ray_dir for leftmost ray (x = 0) and rightmost ray (x = width)
            let ray_dir_x0: f64 = dir_x - plane_x;
            let ray_dir_y0: f64 = dir_y - plane_y;
            let ray_dir_x1: f64 = dir_x + plane_x;
            let ray_dir_y1: f64 = dir_y + plane_y;

            // current y position compared to the screen center (horizon)
            let p: usize = y - SCREEN_HEIGHT / 2;

            // vertical pos of the camera is half the screen height
            let pos_z: f64 = 0.5 * SCREEN_HEIGHT as f64;

            // horizontal distance from the camera to the floor for the current row
            let row_distance: f64 = pos_z / (p as f64);

            // calculates the real-world step vector to add for each x
            let floor_step_x: f64 = row_distance * (ray_dir_x1 - ray_dir_x0) / SCREEN_WIDTH as f64;
            let floor_step_y: f64 = row_distance * (ray_dir_y1 - ray_dir_y0) / SCREEN_WIDTH as f64;

            // real world coords of the leftmost column
            let mut floor_x: f64 = pos_x + row_distance * ray_dir_x0;
            let mut floor_y: f64 = pos_y + row_distance * ray_dir_y0;

            for x in 0..SCREEN_WIDTH {
                // local cell coord are simply the integer parts of the floor coords
                let cell_x: i32 = floor_x as i32;
                let cell_y: i32 = floor_y as i32;

                // get the texture ordinate from the fractional part
                let tx: usize = ((TEX_WIDTH as f64 * (floor_x - cell_x as f64)) as usize) & (TEX_WIDTH - 1);
                let ty: usize = ((TEX_HEIGHT as f64 * (floor_y - cell_y as f64)) as usize) & (TEX_HEIGHT - 1);

                floor_x += floor_step_x;
                floor_y += floor_step_y;

                // draw floor using texture 3 (greystone)
                let mut floor_color: u32 = textures[3][TEX_WIDTH * ty + tx];
                floor_color = (floor_color >> 1) & 0x007F7F7F;
                buffer[y * SCREEN_WIDTH + x] = floor_color;

                // draw ceiling using texture 6 (wood)
                let mut ceil_color: u32 = textures[6][TEX_WIDTH * ty + tx];
                ceil_color = (ceil_color >> 1) & 0x007F7F7F;
                buffer[(SCREEN_HEIGHT - y - 1) * SCREEN_WIDTH + x] = ceil_color;
            }
        }

        // wall ray-casting
        for x in 0..SCREEN_WIDTH {
            let camera_x: f64 = 2.0 * (x as f64) / (SCREEN_WIDTH as f64) - 1.0;
            let ray_dir_x: f64 = dir_x + plane_x * camera_x;
            let ray_dir_y: f64 = dir_y + plane_y * camera_x;

            let mut map_x: usize = pos_x as usize;
            let mut map_y: usize = pos_y as usize;

            let delta_dist_x: f64 = if ray_dir_x == 0.0 { 1e30 } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y: f64 = if ray_dir_y == 0.0 { 1e30 } else { (1.0 / ray_dir_y).abs() };

            let mut side_dist_x: f64;
            let mut side_dist_y: f64;

            let step_x: i32;
            let step_y: i32;

            let mut hit: bool = false;
            let mut side: u8 = 0;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (pos_x - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - pos_x) * delta_dist_x;
            };

            if ray_dir_y > 0.0 {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - pos_y) * delta_dist_y;
            } else {
                step_y = -1;
                side_dist_y = (pos_y - map_y as f64) * delta_dist_y;
            };

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

                if WORLD_MAP[map_x][map_y] > 0 {
                    hit = true;
                }
            };

            let perp_wall_dist: f64 = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };

            z_buffer[x] = perp_wall_dist;

            let line_height: isize = (SCREEN_HEIGHT as f64 / perp_wall_dist) as isize;

            let h: isize = SCREEN_HEIGHT as isize;
            let mut draw_start: isize = -line_height / 2 + h / 2;
            if draw_start < 0 { draw_start = 0; };

            let mut draw_end: isize = line_height / 2 + h / 2;
            if draw_end >= h { draw_end = h - 1; };

            let tex_num: usize = (WORLD_MAP[map_x][map_y] - 1) as usize;

            let mut wall_x: f64 = if side == 0 {
                pos_y + perp_wall_dist * ray_dir_y
            } else {
                pos_x + perp_wall_dist * ray_dir_x
            };
            wall_x -= wall_x.floor();

            let mut tex_x: usize = (wall_x * (TEX_WIDTH as f64)) as usize;

            // Flip texture horizontally based on the side and ray direction
            if side == 0 && ray_dir_x > 0.0 { tex_x = TEX_WIDTH - tex_x - 1; }
            if side == 1 && ray_dir_y < 0.0 { tex_x = TEX_WIDTH - tex_x - 1; }

            // How much to increase the texture coordinate per screen pixel
            let step: f64 = 1.0 * (TEX_HEIGHT as f64) / (line_height as f64);

            // Starting texture coordinate
            let mut tex_pos: f64 = (draw_start - h / 2 + line_height / 2) as f64 * step;

            // Draw the pixels of the stripe
            for y in draw_start..=draw_end {
                // Cast texture coordinate to integer, mask with (TEX_HEIGHT - 1) in case of overflow
                let tex_y: usize = (tex_pos as usize) & (TEX_HEIGHT - 1);
                tex_pos += step;

                // Fetch the color from the texture array
                let mut color: u32 = textures[tex_num][TEX_HEIGHT * tex_y + tex_x];

                // Make color darker for Y-sides (fake lighting)
                if side == 1 {
                    color = (color >> 1) & 0x007F7F7F; // 8355711 in hex
                }

                // Draw to the 1D buffer
                let pixel_index: usize = (y as usize) * SCREEN_WIDTH + x;
                buffer[pixel_index] = color;
            }
        }

        // sort sprites by distance (furthest to nearest)
        // we store pairs of (index, distance) so we can sort the distances but still know which sprite it is
        let mut sprite_order: Vec<(usize, f64)> = sprites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let dist: f64 = (pos_x - s.x).powi(2) + (pos_y - s.y).powi(2);
                (i, dist)
            })
            .collect();

        // sort descending
        (&mut *sprite_order).sort_by(|a, b| b.1.total_cmp(&a.1));

        // calculate the inverse camera matrix
        let inv_det: f64 = 1.0 / (plane_x * dir_y - dir_x * plane_y);

        for (index, _dist) in sprite_order {
            let sprite: &Sprite = &sprites[index];

            // Translate sprite position to relative to camera
            let sprite_x: f64 = sprite.x - pos_x;
            let sprite_y: f64 = sprite.y - pos_y;

            // Transform sprite with the inverse camera matrix
            let transform_x: f64 = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
            let transform_y: f64 = inv_det * (-plane_y * sprite_x + plane_x * sprite_y); // This is the depth (Z)

            // Center of the sprite on screen
            let sprite_screen_x: isize = ((SCREEN_WIDTH as f64 / 2.0) * (1.0 + transform_x / transform_y)) as isize;

            // Calculate height of the sprite on screen
            let sprite_height: isize = (SCREEN_HEIGHT as f64 / transform_y.abs()) as isize;

            // Calculate lowest and highest pixel to fill in current stripe
            let mut draw_start_y: isize = -sprite_height / 2 + SCREEN_HEIGHT as isize / 2;
            if draw_start_y < 0 { draw_start_y = 0; }

            let mut draw_end_y: isize = sprite_height / 2 + SCREEN_HEIGHT as isize / 2;
            if draw_end_y >= SCREEN_HEIGHT as isize { draw_end_y = SCREEN_HEIGHT as isize - 1; }

            // Calculate width of the sprite
            let sprite_width: isize = (SCREEN_HEIGHT as f64 / transform_y.abs()) as isize;
            let mut draw_start_x: isize = -sprite_width / 2 + sprite_screen_x;
            if draw_start_x < 0 { draw_start_x = 0; }

            let mut draw_end_x: isize = sprite_width / 2 + sprite_screen_x;
            if draw_end_x >= SCREEN_WIDTH as isize { draw_end_x = SCREEN_WIDTH as isize - 1; }

            // Loop through every vertical stripe of the sprite on screen
            for stripe in draw_start_x..draw_end_x {
                let mut tex_x: isize = (256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * TEX_WIDTH as isize / sprite_width) / 256;

                // clamp tex_x to prevent out-of-bounds panics
                if tex_x < 0 { tex_x = 0; }
                if tex_x >= TEX_WIDTH as isize { tex_x = TEX_WIDTH as isize - 1; }

                let stripe_usize: usize = stripe as usize;

                // --- Z-BUFFER CHECK ---
                // 1. Is it in front of camera? (transform_y > 0)
                // 2. Is it on the screen? (stripe > 0 && stripe < SCREEN_WIDTH)
                // 3. Is it closer than the wall? (transform_y < z_buffer[stripe])
                if transform_y > 0.0 && stripe > 0 && stripe < SCREEN_WIDTH as isize && transform_y < z_buffer[stripe_usize] {
                    for y in draw_start_y..draw_end_y {
                        let d: isize = (y) * 256 - SCREEN_HEIGHT as isize * 128 + sprite_height * 128; // 256 and 128 factors to avoid floats
                        let mut tex_y: isize = ((d * TEX_HEIGHT as isize) / sprite_height) / 256;

                        // clamp tex_y to prevent out-of-bounds panics
                        if tex_y < 0 { tex_y = 0; }
                        if tex_y >= TEX_HEIGHT as isize { tex_y = TEX_HEIGHT as isize - 1; }

                        let color: u32 = textures[sprite.texture][TEX_WIDTH * (tex_y as usize) + (tex_x as usize)];

                        // Only draw if the pixel is NOT black (0x000000).
                        // This acts as a transparency mask!
                        if (color & 0x00FFFFFF) != 0 {
                            buffer[(y as usize) * SCREEN_WIDTH + stripe_usize] = color;
                        }
                    }
                }
            }
        }

        // input checking
        // move forward if no wall is in front of you
        if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
            if WORLD_MAP[(pos_x + dir_x * move_speed) as usize][pos_y as usize] == 0 {
                pos_x += dir_x * move_speed;
            }
            if WORLD_MAP[pos_x as usize][(pos_y + dir_y * move_speed) as usize] == 0 {
                pos_y += dir_y * move_speed;
            }
        }

        // Move backwards if no wall behind you
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
            if WORLD_MAP[(pos_x - dir_x * move_speed) as usize][pos_y as usize] == 0 {
                pos_x -= dir_x * move_speed;
            }
            if WORLD_MAP[pos_x as usize][(pos_y - dir_y * move_speed) as usize] == 0 {
                pos_y -= dir_y * move_speed;
            }
        }

        // Rotations
        // Rotate to the right
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
            // Calculate sine and cosine once to save processing power
            let cos_rot: f64 = (-rot_speed).cos();
            let sin_rot: f64 = (-rot_speed).sin();

            // Both camera direction and camera plane must be rotated
            let old_dir_x: f64 = dir_x;
            dir_x = dir_x * cos_rot - dir_y * sin_rot;
            dir_y = old_dir_x * sin_rot + dir_y * cos_rot;

            let old_plane_x: f64 = plane_x;
            plane_x = plane_x * cos_rot - plane_y * sin_rot;
            plane_y = old_plane_x * sin_rot + plane_y * cos_rot;
        }

        // Rotate to the left
        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
            let cos_rot: f64 = rot_speed.cos();
            let sin_rot: f64 = rot_speed.sin();

            let old_dir_x: f64 = dir_x;
            dir_x = dir_x * cos_rot - dir_y * sin_rot;
            dir_y = old_dir_x * sin_rot + dir_y * cos_rot;

            let old_plane_x: f64 = plane_x;
            plane_x = plane_x * cos_rot - plane_y * sin_rot;
            plane_y = old_plane_x * sin_rot + plane_y * cos_rot;
        }

        // update the screen
        (&mut window).update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    }
}

fn main() {
    game_window();
}
