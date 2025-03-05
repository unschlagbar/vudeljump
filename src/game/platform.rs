use cgmath::Vector2;

use crate::graphic::InstanceData;

use super::Item;

#[derive(Debug)]
pub struct Platform {
    pub pos: Vector2<f32>,
    pub size: Vector2<f32>,
    pub id: u8,
    pub direction: f32,
    pub item: Option<Item>,
}

impl Platform {
    pub const fn new(pos: Vector2<f32>, size: Vector2<f32>, id: u8, item: Option<Item>) -> Self {
        Self { pos, size, id, direction: 1.0, item: item }
    }

    #[inline]
    pub fn update(&mut self, delta_time: f32) {
        if self.id != 1 { return };

        self.pos.x += 100.0 * self.direction * delta_time;

        if let Some(item) = &mut self.item {
            item.pos.x += 100.0 * self.direction * delta_time;
        }

        // Grenzen für die Bewegung
        if self.pos.x >= 330.0 {
            self.direction = -1.0; // Wechsle die Richtung nach links
        } else if self.pos.x <= 10.0 {
            self.direction = 1.0;  // Wechsle die Richtung nach rechts
        }
    }

    pub fn get_instance(&self, vec: &mut Vec<InstanceData>) {
        let mut pos = self.pos;
        pos.y *= -1.0;

        //((high as u32) << 16) | (low as u32)
        let uv_start;
        let uv_end;

        if self.id == 0 {
            uv_start = 0;
            uv_end = (6 << 16) | 30; 
        } else {
            uv_start = 8 << 16;
            uv_end = (6 << 16) | 30; 
        }
        vec.push(InstanceData::new(pos, self.size, 0, uv_start, uv_end ));

        if let Some(item) = &self.item {
            vec.push(item.get_instance());
        }
    }
}