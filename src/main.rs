use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

// constants
const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 480;
const MAP_WIDTH: usize = 24;
const MAP_HEIGHT: usize = 24;

// world map
static WORLD_MAP: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,2,2,2,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
    [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,3,0,0,0,3,0,0,0,1],
    [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,2,2,0,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,0,0,0,5,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
];

// main functions
fn game_window() {
    let mut buffer: Vec<u32> = vec![0; MAP_WIDTH * MAP_HEIGHT];

    let mut window: Window = Window::new(
        "EyeGore",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("Failed to create window: {}", e);
    });

    window.set_target_fps(60);

    let mut current_time: Instant = Instant::now();

    let mut pos_x: f64 = 22.00;
    let mut pos_y: f64 = 12.00;
    let mut dir_x: f64 = -1.00;
    let mut dir_y: f64 = 0.00;
    let mut plane_x: f64 = 0.00;
    let mut plane_y: f64 = 0.66;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Timing
        let new_time: Instant = Instant::now();
        let frame_time: f64 = new_time.duration_since(current_time).as_secs_f64();
        current_time = new_time;

        let move_speed: f64 = frame_time * 5.0;
        let rot_speed: f64 = frame_time * 3.0;

        // clear the screen with black pixels
        buffer.fill(0x00000000);

        // RAY-CASTING LOOP BELOW
        for x in 0..SCREEN_WIDTH {
            let camera_x: f64 = 2.0 * (x as f64) / (SCREEN_WIDTH as f64) - 1.0;
            let ray_dir_x: f64 = dir_x + plane_x * camera_x;
            let ray_dir_y: f64 = dir_y + plane_y * camera_x;

            let mut map_x: usize = pos_x as usize;
            let mut map_y: usize = pos_y as usize;

            let mut delta_dist_x: f64 = if ray_dir_x == 0.0 { 1e30 } else { (1.0 / ray_dir_x).abs() };
            let mut delta_dist_y: f64 = if ray_dir_y == 0.0 { 1e30 } else { (1.0 / ray_dir_y).abs() };

            let mut side_dist_x: f64;
            let mut side_dist_y: f64;

            let mut step_x: i32;
            let mut step_y: i32;

            let mut hit: bool = false;
            let mut side: u8 = 0;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (pos_x - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - pos_x) * delta_dist_x;
            };

            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (pos_y - map_y as f64) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - pos_y) * delta_dist_y;
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

            let line_height: isize = (SCREEN_HEIGHT as f64 / perp_wall_dist) as isize;

            let h: isize = SCREEN_HEIGHT as isize;
            let mut draw_start: isize = -line_height / 2 + h / 2;
            if draw_start < 0 { draw_start = 0; };

            let mut draw_end: isize = line_height / 2 + h / 2;
            if draw_end >= h { draw_end = h - 1; };

            let mut color: u32 = match WORLD_MAP[map_x][map_y] {
                1 => 0x00FF0000, // Red
                2 => 0x0000FF00, // Green
                3 => 0x000000FF, // Blue
                4 => 0x00FFFFFF, // White
                _ => 0x00FFFF00, // Yellow
            };

            if side == 1 {
                color = (color >> 1) & 0x007F7F7F
            }

            for y in draw_start..=draw_end {
                let pixel_index: usize = (y as usize) * SCREEN_WIDTH + x;
                buffer[pixel_index] = color;
            }
        }

        if window.is_key_down(Key::Up) {
            if WORLD_MAP[(pos_x + dir_x * move_speed) as usize][pos_y as usize] == 0 {
                pos_x += dir_x * move_speed;
            }
            if WORLD_MAP[pos_x as usize][(pos_y + dir_y * move_speed) as usize] == 0 {
                pos_y += dir_y * move_speed;
            }
        }

        // Move backwards if no wall behind you
        if window.is_key_down(Key::Down) {
            if WORLD_MAP[(pos_x - dir_x * move_speed) as usize][pos_y as usize] == 0 {
                pos_x -= dir_x * move_speed;
            }
            if WORLD_MAP[pos_x as usize][(pos_y - dir_y * move_speed) as usize] == 0 {
                pos_y -= dir_y * move_speed;
            }
        }

        // Rotate to the right
        if window.is_key_down(Key::Right) {
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
        if window.is_key_down(Key::Left) {
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
        window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    }
}

fn main() {
    game_window();
}
