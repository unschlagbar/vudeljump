use std::{cell::RefCell, rc::Rc};

use cgmath::Vector2;
use iron_oxide::ui::{Text, UiElement, UiState, UiType};
use rand::Rng;

pub use crate::game::Player;
use crate::graphic::InstanceData;

use super::{Item, Platform};

#[repr(C)]
#[derive(Debug)]
pub struct World {
    pub platforms: Vec<Platform>,
    //pub enemies: Vec<()>,
    pub player: Player,
    pub view_start: u32,
    pub view_end: u32,
    pub current_view: f32,
    pub gen_heigt: f32,
    pub platform_density: f32,
    pub score: u32,
    pub ui: Rc<RefCell<UiState>>,
    dead: bool,
}

impl World {

    pub fn create(ui: Rc<RefCell<UiState>>) -> Self {
        let mut platforms = Vec::with_capacity(30);
        platforms.push(Platform::new(Vector2 { x: 170.0, y: 50.0 }, Vector2 { x: 60.0, y: 12.0 }, 0, None));

        Self {
            platforms,
            //enemies: Vec::with_capacity(10),
            player: Player::create(Vector2 { x: 185.0, y: 0.0 }),
            view_start: 0,
            view_end: 600,
            current_view: 0.0,
            gen_heigt: 75.0,
            platform_density: 0.0,
            score: 0,
            ui,
            dead: false,
        }
    }

    pub fn get_instances(&self) -> Vec<InstanceData> {
        let mut vec = Vec::with_capacity(20);

        for platform in &self.platforms {
            platform.get_instance(&mut vec);
        }
        vec.push(self.player.get_instance());

        vec
    }

    #[inline]
    fn remove_platforms_below(&mut self) {
        self.platforms.retain(|platform| platform.pos.y > self.view_start as f32); // Behalte nur Plattformen oberhalb des Bildschirms
    }

    fn generate_platforms(&mut self) {
        let y_spacing = 30.0;
        let max_x = 320.0;
        let hardness = (self.score as f32).sqrt() / 60.0;           // Rechter Rand des Bildschirms (Beispiel: Bildschirmbreite)

        let mut rng = rand::thread_rng();

        // Generiere neue Plattformen, wenn der Spieler nach oben gesprungen ist
        while self.gen_heigt < self.view_end as f32 {

            if rng.gen_range(0.0..1.3) <= hardness * self.platform_density {
                self.gen_heigt += y_spacing;
                self.platform_density -= 1.0 / 3.0;
                continue;
            }

            let y_position = self.gen_heigt + y_spacing;

            // Zufällige X-Position im sichtbaren Bereich
            let x_position = rng.gen_range(0.0..max_x);

            let mut platform;

            if rng.gen_range(0.0..9.0) <= hardness.clamp(0.0, 5.0) {
                platform = Platform {
                    pos: Vector2 { x: x_position, y: y_position },
                    size: Vector2 { x: 60.0, y: 12.0 },  // Beispielsgröße
                    id: 1,
                    direction: 1.0,
                    item: None
                }
            } else {
                platform = Platform {
                    pos: Vector2 { x: x_position, y: y_position },
                    size: Vector2 { x: 60.0, y: 12.0 },
                    id: 0,
                    direction: 0.0,
                    item: None
                };
            }

            if rng.gen_range(1..20) == 1 + hardness.min(3.0) as u32 {
                platform.item = Some(Item { pos: Vector2 { x: x_position + rng.gen_range(0.0..40.0), y: y_position + 15.0 }, size: Vector2 { x: 20.0, y: 20.0 }, typ: 1 });
            }

            self.platform_density = 1.0;
            self.platforms.push(platform);
            self.gen_heigt += y_spacing;
        }
    }

    pub fn update(&mut self, delta_time: f32) {

        if delta_time > 1.0 || self.dead {
            return;
        }

        let prev_pos = self.player.pos;

        self.player.update(delta_time);

        if self.player.pos.y >= self.score as f32 {
            self.score = self.player.pos.y as u32;
            let mut ui = self.ui.borrow_mut();
            let score_element = ui.get_element(vec![0, 0]).unwrap();
            match &score_element.inherit {
                UiType::Text(text) => unsafe { (text as *const Text as *mut Text).as_mut().unwrap_unchecked().set_text((score_element as *const UiElement as *mut UiElement).as_mut().unwrap_unchecked(), &self.score.to_string()) },
                _ => (),
            };
            ui.dirty = true;
        } else if self.player.velocity.y < 0.0 && self.player.pos.y < self.view_start as f32 {
            self.dead = true;
            let mut ui = self.ui.borrow_mut();
            {
                let dead_text = unsafe { ui.get_element_mut(vec![1]).unwrap() };
                dead_text.visible = true;
                dead_text.dirty = true;
            }
            {
                let restart_button = unsafe { ui.get_element_mut(vec![2]).unwrap() };
                restart_button.visible = true;
                restart_button.dirty = true;
            }
            ui.dirty = true;
        }

        self.smooth_view(delta_time, 15.0);

        let step = self.player.pos.y - 300.0;
        if step > self.view_start as f32 {
            self.view_end = self.view_end - self.view_start + step as u32;
            self.view_start = step as u32;
            self.remove_platforms_below();
        }

        self.generate_platforms();


        for platform in &mut self.platforms {
            platform.update(delta_time);
            if let Some(item) = &platform.item {
                if self.player.collides_with_item(item) {
                    if item.typ == 1{
                        self.player.jump_with_strenght(800.0);
                    }
                    break;
                }
            } 
            
            if self.player.collides_with_platform(prev_pos, platform) {
                self.player.jump();
                break;
            }
        }
    }

    #[inline]
    pub fn smooth_view(&mut self, delta_time: f32, smoothing_factor: f32) {
        self.current_view += (self.view_start as f32 - self.current_view) * smoothing_factor * delta_time;
    }

    #[no_mangle]
    pub fn restart(&mut self, ui: &mut UiState, _: &mut UiElement) {
        
        self.dead = false;
        self.score = 0;
        self.gen_heigt = 75.0;
        self.platforms.clear();
        let view_scope = self.view_end - self.view_start;
        self.view_start = 0;
        self.view_end = view_scope;
        self.player = Player::create(Vector2 { x: 185.0, y: 0.0 });

        self.platforms.push(Platform::new(Vector2 { x: 170.0, y: 50.0 }, Vector2 { x: 60.0, y: 12.0 }, 0, None));

        {
            let dead_text = unsafe { ui.get_element_mut(vec![1]).unwrap() };
            dead_text.visible = false;
            dead_text.dirty = true;
        }
        {
            let resart_button = unsafe { ui.get_element_mut(vec![2]).unwrap() };
            resart_button.visible = false;
            resart_button.dirty = true;
        }

        ui.dirty = true;
    }


}