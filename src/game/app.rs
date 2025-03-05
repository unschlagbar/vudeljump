
use std::{cell::RefCell, mem::{MaybeUninit, ManuallyDrop}, ops::Not, rc::Rc, thread::sleep, time::{Duration, Instant}};

use cgmath::Vector2;
use iron_oxide::{primitives::Vec2, ui::{UiEvent, UiState}};
use log::info;
use winit::{application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::{self, ElementState, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, keyboard::KeyCode, window::WindowId};

use crate::graphic::VulkanRender;

use super::{states::build_main, World};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 600;
const FPS: f32 = 144.0;
const FRAME_TIME: f32 = 1.0 / FPS;

#[allow(dead_code)]
pub struct App {
    pub renderer: Rc<RefCell<ManuallyDrop<VulkanRender>>>,
    pub init: bool,
    pub cursor_pos: PhysicalPosition<f64>,
    pub world: World,
    pub time: Instant,
    pub ui: Rc<RefCell<UiState>>,
    pub last_cursor_location: Vec2,
    pub touch_id: u64,
    pub mouse_pressed: bool,
    pub sim_speed: f32,
}

impl App {
    #[allow(dead_code)]
    #[inline]
    pub fn run() -> Self {
        #[allow(invalid_value)]
        let renderer= Rc::new(RefCell::new(ManuallyDrop::new(unsafe { MaybeUninit::uninit().assume_init() })));
        let ui: Rc<RefCell<UiState>> = Rc::new(RefCell::new(build_main()));
        let world = World::create(ui.clone());

        Self {
            renderer,
            init: false,
            cursor_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            world, time: Instant::now(),
            ui,
            last_cursor_location: Vec2::zero(),
            touch_id: 0,
            mouse_pressed: false,
            sim_speed: 1.0,
        }
    }
}

impl ApplicationHandler for App {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if !self.init {
            return;
        }
        let mut renderer = self.renderer.borrow_mut();

        match event {
            event::WindowEvent::CursorMoved { device_id: _,  position } => {

                {
                    let mut ui = renderer.ui_state.borrow_mut();
                    ui.update_cursor(Vec2::new(renderer.window_size.width as f32, renderer.window_size.height as f32), Vec2::new(position.x as f32, position.y as f32), UiEvent::Move);
                }

                self.cursor_pos = position;
            },
            event::WindowEvent::MouseInput { device_id: _, state, button } => {
                match button {
                    MouseButton::Left => {
                        self.mouse_pressed = state == ElementState::Pressed;
                        renderer.ui_state.borrow_mut().update_cursor(Vec2::new(renderer.window_size.width as f32, renderer.window_size.height as f32), Vec2::new(self.cursor_pos.x as f32, self.cursor_pos.y as f32), 
                            match state {
                                ElementState::Pressed => UiEvent::Press,
                                ElementState::Released => UiEvent::Release,
                            }
                        );
                    }
                    _ => ()
                }
            },
            event::WindowEvent::Touch(touch) => {
                let cursor_pos = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                match touch.phase {
                    event::TouchPhase::Started => {
                        if touch.id != 0 || self.touch_id != touch.id {return;}
                        self.touch_id = touch.id;
                        renderer.ui_state.borrow_mut().update_cursor(Vec2::new(renderer.window_size.width as f32, renderer.window_size.height as f32), cursor_pos, UiEvent::Press);
                        self.last_cursor_location = cursor_pos;
                    },
                    event::TouchPhase::Moved => {
                        let in_ui = renderer.ui_state.borrow_mut().update_cursor(Vec2::new(renderer.window_size.width as f32, renderer.window_size.height as f32), cursor_pos, UiEvent::Move);
                        
                        if in_ui < 2 {
                            let diff = cursor_pos - self.last_cursor_location;
                            self.world.player.pos += Vector2::new(diff.x, 0.0);
                        }
                        
                        self.last_cursor_location = cursor_pos;
                    },
                    //ended
                    _ => {
                        self.touch_id = 0;
                        renderer.ui_state.borrow_mut().update_cursor(Vec2::new(renderer.window_size.width as f32, renderer.window_size.height as f32), cursor_pos, UiEvent::Release);
                    }
                }
            },
            event::WindowEvent::RedrawRequested => {
                drop(renderer);
                let time_stamp = self.time.elapsed().as_secs_f32();
                if time_stamp > FRAME_TIME * 0.92 {
                    self.time = Instant::now();
                    self.world.update(self.sim_speed * time_stamp);
                    let mut renderer = self.renderer.borrow_mut();
                    renderer.draw_frame();
                } else {
                    sleep(Duration::from_nanos(500_000));
                };

            },
            event::WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => {

                        match key_code {
                            KeyCode::F1 => {
                                if event.state.is_pressed() {
                                    {
                                        let mut value = renderer.ui_state.borrow_mut();
                                        value.visible = value.visible.not();
                                        value.dirty = true;
                                    }
                                }
                            },
                            KeyCode::KeyX => {
                                if event.state.is_pressed() {
                                    if self.sim_speed == 0.0 {
                                        self.sim_speed = 1.0;
                                    } else {
                                        self.sim_speed = 0.0;
                                    }
                                }
                            },
                            KeyCode::KeyM => {
                                if event.state.is_pressed() {
                                    renderer.renderer = !renderer.renderer;
                                }
                            },
                            KeyCode::KeyA => {
                                if event.state.is_pressed() {
                                    self.world.player.movement = -1;
                                } else if self.world.player.movement == -1 {
                                    self.world.player.movement = 0;
                                }
                            },
                            KeyCode::KeyD => {
                                if event.state.is_pressed() {
                                    self.world.player.movement = 1;
                                } else if self.world.player.movement == 1 {
                                    self.world.player.movement = 0;
                                }
                            }
                            _ => ()
                        }
                    },
                    _ => ()
                }
            },
            event::WindowEvent::Resized(new_size) => {
                info!("resized");
                if !self.init {
                    return;
                }
                let size = renderer.window.inner_size();
                if new_size != size || new_size == renderer.window_size {
                    return;
                }
                renderer.recreate_swapchain(size);
                renderer.update_ui(new_size);
                renderer.window.request_redraw();
                self.world.view_end = self.world.view_start + new_size.height;
            },
            event::WindowEvent::CloseRequested => {
                event_loop.exit();
                unsafe { renderer.base.device.device_wait_idle().unwrap_unchecked() };
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.init {
            return;
        }
        self.renderer.borrow().window.request_redraw();
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        println!("suspended");
        if !self.init {
            return;
        }
        self.init = false;
        let mut renderer = self.renderer.borrow_mut();
        unsafe { renderer.base.device.device_wait_idle().unwrap_unchecked(); };
        renderer.destroy();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    }
    
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init = true;
        println!("resumed");
        let window_attributes = winit::window::Window::default_attributes().with_title("Vudeljump").with_inner_size(PhysicalSize {width: WIDTH, height: HEIGHT}).with_min_inner_size(PhysicalSize {width: WIDTH, height: HEIGHT});
        let window = event_loop.create_window(window_attributes).unwrap();
        let mut renderer = self.renderer.borrow_mut();
        *renderer = ManuallyDrop::new(VulkanRender::create(window, &self.world));

        self.ui.borrow_mut().init_graphics(&renderer.base, &renderer.window_size, renderer.render_pass, &renderer.ui_descriptor_set_layout);
        let window_size = renderer.window_size;
        renderer.update_ui(window_size);

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        self.time = Instant::now();
        renderer.draw_frame();

        self.world.view_end = self.world.view_start + window_size.height;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("exiting");
        if !self.init {
            return;
        }
        self.init = false;
    }
}