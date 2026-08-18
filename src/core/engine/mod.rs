pub mod render;
pub mod update;

use crate::core::config::{GameConfig, load_config};
use crate::core::input::InputState;
use crate::utils::helpers::{create_window, load_texture};
use crate::world::map::CartesianPos;
use crate::world::player::Player;
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

// main Engine struct

/// Represents the entirety of the game's Engine, wraps most of the other modules' methods
pub struct Engine {
    /// configuration struct
    config: GameConfig,
    /// designated window
    window: Window,
    /// pixel-by-pixel rendering buffer
    buffer: Vec<u32>,
    z_buffer: Vec<f64>,
    /// Public debug mode option
    pub show_debug: bool,
    /// Current level
    pub current_level_idx: usize,
    /// textures rendering buffer
    textures: Vec<Vec<u32>>,
    /// player struct
    player: Player,
    /// input handling struct
    input: InputState,
    /// Public pause game check
    pub is_paused: bool,
    /// sprite entitie buffer
    sprite_buffer: Vec<(usize, f64)>,
    /// multiuse debug string
    debug_string: String,
}
impl Engine {
    // Engine constructor

    /// Engine's constructor
    ///
    /// # Arguments
    ///
    /// * `filepath` - the .ron config file to load.
    ///
    /// # Returns
    /// * `Self` - called by other methods.
    pub fn new(filepath: &str) -> Self {
        let config = load_config(filepath);

        let mut window = create_window(&config);

        window.set_cursor_visibility(false);

        // simple check for levels
        if config.levels.is_empty() {
            panic!("Configuration file has no levels!");
        }

        let mut textures: Vec<Vec<u32>> = Vec::with_capacity(config.levels[0].textures.len());

        // always starts with level 0
        for tx in &config.levels[0].textures {
            textures.push(load_texture(tx));
        }

        let buffer_size = config.scr_width * config.scr_height;

        let z_buffer_size = config.scr_width;

        let starting_player = Player::new(
            CartesianPos {
                x: config.levels[0].player_start_x,
                y: config.levels[0].player_start_y,
            },
            CartesianPos { x: -1.0, y: 0.0 },
            CartesianPos { x: 0.0, y: 0.66 },
        );

        Self {
            config,
            window,
            buffer: vec![0; buffer_size],
            z_buffer: vec![0.0; z_buffer_size],
            textures,
            current_level_idx: 0,
            show_debug: false,
            player: starting_player,
            input: InputState::new(),
            is_paused: false,
            sprite_buffer: Vec::new(),
            debug_string: String::with_capacity(64),
        }
    }

    // Engine runner, wraps all other methods

    /// Engine run wrapper
    ///
    /// when calling this the Engine will call all other methods according to the .ron config,
    /// this is what you want to call to run the game.
    pub fn run(&mut self) {
        let mut current_time = Instant::now();

        while self.window.is_open() && !self.window.is_key_down(Key::Key0) {
            // calculating delta time
            let new_time = Instant::now();
            // frame_time is time relative to the game run time (delta)
            let frame_time = new_time.duration_since(current_time).as_secs_f64();
            current_time = new_time;

            // autopause on lost focus
            if !self.window.is_active() && !self.is_paused {
                self.is_paused = true;
            }

            // input handling
            self.handle_input(frame_time);

            // entity updating
            if !self.is_paused {
                self.update_entities(frame_time);
            }

            // floor and ceiling rendering math
            self.render_floor_ceiling();

            // wall rendering math
            self.render_walls();

            // sprite rendering
            self.render_sprites();

            // debug overlay rendering
            if self.show_debug {
                self.render_debug_overlay(frame_time);
            }

            // draws a large "PAUSED" in the center of the screen
            if self.is_paused {
                let center_x = (self.config.scr_width / 2) - 96;
                let center_y = (self.config.scr_height / 2) - 16;

                Engine::draw_text(
                    &mut self.buffer,
                    self.config.scr_width,
                    self.config.scr_height,
                    "PAUSED",
                    center_x,
                    center_y,
                    4,
                );
                Engine::draw_text(
                    &mut self.buffer,
                    self.config.scr_width,
                    self.config.scr_height,
                    "PRESS Esc TO RESUME",
                    center_x - 16,
                    center_y + 40,
                    2,
                );
            }

            // finally, update the window
            self.window
                .update_with_buffer(&self.buffer, self.config.scr_width, self.config.scr_height)
                .unwrap();
        }
    }
}
