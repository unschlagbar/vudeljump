use cgmath::Vector2;

use crate::graphic::InstanceData;

#[derive(Debug)]
pub struct Item {
    pub pos: Vector2<f32>,
    pub size: Vector2<f32>,
    pub typ: u8,
}

impl Item {
    pub fn get_instance(&self) -> InstanceData {
        let mut pos = self.pos;
        pos.y *= -1.0;

        //((high as u32) << 16) | (low as u32)
        let uv_start = 160;
        let uv_end = (7 << 16) | 9; 
        InstanceData::new(pos, self.size, 0, uv_start, uv_end )
    }
}