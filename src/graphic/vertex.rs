
use ash::vk;

use super::instance::InstanceData;


#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {}

impl Vertex {
    pub const GET_BINDING_DESCRIPTION: [vk::VertexInputBindingDescription; 1] = [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<InstanceData>() as _,
            input_rate: vk::VertexInputRate::INSTANCE,
        },
    ];

    pub const GET_ATTRIBUTE_DESCRIPTIONS: [vk::VertexInputAttributeDescription; 5] = [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32_SFLOAT,
            offset: 8,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32_UINT,
            offset: 16,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 3,
            format: vk::Format::R32_UINT,
            offset: 20,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 4,
            format: vk::Format::R32_UINT,
            offset: 24,
        },
    ];
}