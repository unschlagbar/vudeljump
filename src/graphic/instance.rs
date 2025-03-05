use std::mem::offset_of;

use ash::vk;
use cgmath::Vector2;

#[allow(dead_code)]
#[derive(Debug)]
pub struct InstanceData {
    id: u32,
    position: Vector2<f32>,
    size: Vector2<f32>,
    uv_start: u32,
    uv_size: u32,
}

impl InstanceData {
    #[allow(unused)]
    pub const GET_ATTRIBUTE_DESCRIPTIONS: [vk::VertexInputAttributeDescription; 5] = [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 3,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(InstanceData, position) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 4,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(InstanceData, size) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 5,
            format: vk::Format::R32_UINT,
            offset: offset_of!(InstanceData, id) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 6,
            format: vk::Format::R32_UINT,
            offset: offset_of!(InstanceData, uv_start) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 7,
            format: vk::Format::R32_UINT,
            offset: offset_of!(InstanceData, uv_size) as _,
        },
    ];
    pub const fn new(position: Vector2<f32>, size: Vector2<f32>, id: u32, uv_start: u32, uv_size: u32) -> Self {
        Self { position, size, id, uv_start, uv_size }
    }
}