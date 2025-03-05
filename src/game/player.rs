
use cgmath::Vector2;

use crate::graphic::InstanceData;

use super::{Item, Platform};

#[derive(Debug)]
pub struct Player {
    pub pos: Vector2<f32>,
    pub size: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub movement: i8,
}

impl Player {
    
    pub const fn create(pos: Vector2<f32>) -> Self {
        Self { pos, size: Vector2 { x: 20.0, y: 40.0 }, velocity: Vector2 { x: 0.0, y: 460.0 }, movement: 0 }
    }

    pub fn get_instance(&self) -> InstanceData {
        let mut pos = self.pos;
        pos.y *= -1.0;

        //((high as u32) << 16) | (low as u32);
        let uv_start = 32;
        let uv_end = (256 << 16) | 128; 
        InstanceData::new(pos, self.size, 0, uv_start, uv_end )
    }

    pub fn update(&mut self, delta_time: f32) {
        self.velocity.y -= 600.0 * delta_time;
        self.pos += self.velocity * delta_time;
        self.pos.x = (self.pos.x + self.movement as f32 * 500.0 * delta_time).clamp(0.0, 380.0);
    }

    pub fn collides_with_platform(&self, prev_pos: Vector2<f32>, platform: &Platform) -> bool {
        // Ignoriere Kollisionen, wenn der Spieler nach oben springt
        if self.velocity.y > 0.0 {
            return false;
        }
    
        // Spieler-Koordinaten (jetzt und vorher)
        let player_bottom = self.pos.y - self.size.y;
        let prev_player_bottom = prev_pos.y - self.size.y;
    
        // Plattform-Koordinaten
        let platform_right = platform.pos.x + platform.size.x;
        let platform_left = platform.pos.x;
        let platform_top = platform.pos.y;
    
        // Überprüfung der horizontalen Überlappung (Spieler und Plattform überlappen sich in der Breite)
        let horizontal_overlap = self.pos.x + self.size.x > platform_left && self.pos.x < platform_right;
    
        // Überprüfen, ob der Spieler die Plattform im vertikalen Raum durchquert hat
        let vertical_pass_through = prev_player_bottom + 1.0 > platform_top && player_bottom <= platform_top + 1.0;
    
        // Kollision tritt nur auf, wenn sowohl die horizontale als auch die vertikale Durchquerung vorliegt
        horizontal_overlap && vertical_pass_through
    }
    
    

    pub fn collides_with_item(&self, item: &Item) -> bool {

        if self.velocity.y > 0.0 {
            return false;
        }

        let player_right = self.pos.x + self.size.x;
        let player_left = self.pos.x;
        let player_bottom = self.pos.y - self.size.y;
        let platform_right = item.pos.x + item.size.x;
        let platform_left = item.pos.x;
        let platform_top = item.pos.y;

        // Überprüfung der horizontalen Überlappung (Spieler und Plattform überlappen sich in der Breite)
        let horizontal_overlap = player_right > platform_left && player_left < platform_right;

        // Überprüfung, ob der Spieler von oben auf die Plattform trifft
        let vertical_overlap = player_bottom >= platform_top && player_bottom <= platform_top + 5.0; // "Toleranzbereich" der Kollision

        // Kollision tritt nur auf, wenn der Spieler fällt und auf die Plattform von oben trifft
        horizontal_overlap && vertical_overlap
    }

    pub fn jump(&mut self) {
        self.velocity.y = 460.0;
    }

    pub fn jump_with_strenght(&mut self, strenght: f32) {
        self.velocity.y = strenght;
    }
}