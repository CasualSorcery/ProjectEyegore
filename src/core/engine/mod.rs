pub mod render;
pub mod update;

use crate::core::config::{load_config, GameConfig};
use crate::core::input::InputState;
use crate::utils::helpers::load_texture;
use crate::world::map::CartesianPos;
use crate::world::player::Player;

use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

// main Engine struct
/// Represents the entirety of the game's Engine, wraps most of the other modules' methods
pub struct Engine {
    /// configuration struct
    config: GameConfig,
    /// designated window, wrapped by Rc (reference counted)
    window: Option<Rc<Window>>,
    /// pixel-by-pixel rendering buffer
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
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
    /// used for delta time
    last_frame_time: Instant,
}
impl Engine {
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
            window: None,
            surface: None,
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
            last_frame_time: Instant::now(),
        }
    }
}
impl ApplicationHandler for Engine {
    // as the loop is resumed
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // if there's no window, create one
        if self.window.is_none() {
            let mut window_attributes = Window::default_attributes()
                .with_title(&self.config.name)
                .with_inner_size(PhysicalSize::new(
                    self.config.scr_width as f64,
                    self.config.scr_height as f64,
                ));

            // fullscreen check
            if self.config.fullscreen {
                window_attributes = window_attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
            }

            // wrap the window in rc
            let window = Rc::new(event_loop.create_window(window_attributes).unwrap());

            // locks the cursor inside the window
            window.set_cursor_visible(false);
            let _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));

            // initialize softbuffer using the cloned rc pointers
            let context = Context::new(window.clone()).unwrap();
            let mut surface = Surface::new(&context, window.clone()).unwrap();

            // the window surface, where the pixels will be drawn upon
            surface
                .resize(
                    NonZeroU32::new(self.config.scr_width as u32).unwrap(),
                    NonZeroU32::new(self.config.scr_height as u32).unwrap(),
                )
                .unwrap();

            // assign the values to the variables
            self.window = Some(window);
            self.surface = Some(surface);
            self.last_frame_time = Instant::now();
        }
    }

    // any window event (button press, resizing, etc.)
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            // if a close window has been requested, close window
            WindowEvent::CloseRequested => event_loop.exit(),

            // redraw event
            WindowEvent::RedrawRequested => {
                if self.window.is_none() || self.surface.is_none() {
                    return;
                }

                // delta time calculation
                let new_time = Instant::now();
                let frame_time = new_time.duration_since(self.last_frame_time).as_secs_f64();
                self.last_frame_time = new_time;

                // input takes precedent over other events
                self.handle_input(frame_time);

                // only update entities if unpaused
                if !self.is_paused {
                    self.update_entities(frame_time);
                }

                // all other renders
                self.render_floor_ceiling();
                self.render_walls();
                self.render_sprites();

                // debug render
                if self.show_debug {
                    self.render_debug_overlay(frame_time);
                }

                // pause screen
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

                // draw to softbuffer
                if let Some(surface) = &mut self.surface {
                    let win_size = self.window.as_ref().unwrap().inner_size();
                    let out_w = win_size.width as usize;
                    let out_h = win_size.height as usize;

                    // don't draw if the window is minimized
                    if out_w > 0 && out_h > 0 {
                        // resize the softbuffer surface to match the monitor
                        surface.resize(
                            NonZeroU32::new(win_size.width).unwrap(),
                            NonZeroU32::new(win_size.height).unwrap(),
                        ).unwrap();

                        let mut screen_buffer = surface.buffer_mut().unwrap();

                        let in_w = self.config.scr_width;
                        let in_h = self.config.scr_height;

                        // nearest neighbor software scaling
                        for y in 0..out_h {
                            let src_y = (y * in_h) / out_h;
                            let row_offset = src_y * in_w;
                            let out_row_offset = y * out_w;

                            for x in 0..out_w {
                                let src_x = (x * in_w) / out_w;
                                // map the pixel from the 640x480 buffer to any res screen
                                screen_buffer[out_row_offset + x] = self.buffer[row_offset + src_x];
                            }
                        }
                        screen_buffer.present().unwrap();
                    }
                }
                // immediately request next frame redraw so windows doesn't freeze
                self.window.as_ref().unwrap().request_redraw();
            }

            // keyboard input handling
            WindowEvent::KeyboardInput {
                event: kb_event, ..
            } => {
                // reads the physical state of keyboard keys
                if let PhysicalKey::Code(keycode) = kb_event.physical_key {
                    if kb_event.state == ElementState::Pressed {
                        // F12 closes the game
                        if keycode == KeyCode::F12 {
                            event_loop.exit();
                        }

                        // toggles fullscreen (if not repeated key)
                        if keycode == KeyCode::F11 {
                            self.config.fullscreen = !self.config.fullscreen;

                            if let Some(win) = &self.window {
                                if self.config.fullscreen {
                                    win.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                } else {
                                    win.set_fullscreen(None); // None = Windowed

                                }
                            }
                        }

                        // Esc pauses (if not repeated key)
                        if keycode == KeyCode::Escape && !kb_event.repeat {
                            self.is_paused = !self.is_paused;
                            if let Some(win) = &self.window {
                                if self.is_paused {
                                    win.set_cursor_visible(true);
                                    let _ = win.set_cursor_grab(CursorGrabMode::None);
                                } else {
                                    win.set_cursor_visible(false);
                                    let _ = win
                                        .set_cursor_grab(CursorGrabMode::Confined)
                                        .or_else(|_| win.set_cursor_grab(CursorGrabMode::Locked));
                                }
                            }
                        }

                        // E changes weapon (if not paused, and not repeated key)
                        if keycode == KeyCode::KeyE && !self.is_paused && !kb_event.repeat {
                            self.player.inventory.change_weapon();
                        }
                        // F3 shows debug overlay
                        if keycode == KeyCode::F3 && !kb_event.repeat {
                            self.show_debug = !self.show_debug;
                        }

                        // adds to the keys_held hash map
                        self.input.keys_held.insert(PhysicalKey::Code(keycode));
                    } else {
                        // removes from the keys_held hash map
                        self.input.keys_held.remove(&PhysicalKey::Code(keycode));
                    }
                }
            }

            // left mouse button shoots
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    self.input.left_mouse_down = state == ElementState::Pressed;
                }
            }

            // pauses if screen is unfocused
            WindowEvent::Focused(focused) if !focused && !self.is_paused => {
                self.is_paused = true;
                if let Some(win) = &self.window {
                    win.set_cursor_visible(true);
                    let _ = win.set_cursor_grab(CursorGrabMode::None);
                }
            }
            // else any other cases
            _ => {}
        }
    }

    // raw input from devices events
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // gets the mouse raw input
        if let DeviceEvent::MouseMotion { delta } = event
            && !self.is_paused
        {
            self.input.mouse_dx += delta.0;
            self.input.mouse_dy += delta.1;
        }
    }
}