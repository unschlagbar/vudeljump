use std::{ffi::c_void, mem::MaybeUninit, ptr::null_mut};

use ash::vk::{self};
use iron_oxide::graphics::{self, VkBase};

use super::{vulkan_render::MAXFRAMESINFLIGHT, UniformBufferObject};

pub fn create_uniform_buffers(base: &VkBase) -> ([graphics::Buffer; MAXFRAMESINFLIGHT], [*mut c_void; MAXFRAMESINFLIGHT]) {
    let buffer_size = std::mem::size_of::<UniformBufferObject>() as u64;

    #[allow(invalid_value)]
    let mut uniform_buffers: [graphics::Buffer; MAXFRAMESINFLIGHT] = [unsafe { MaybeUninit::uninit().assume_init() }; MAXFRAMESINFLIGHT];
    let mut mapped: [*mut c_void; MAXFRAMESINFLIGHT] = [null_mut(); MAXFRAMESINFLIGHT];

    for i in 0..MAXFRAMESINFLIGHT {
        uniform_buffers[i] = graphics::Buffer::create(base, buffer_size, vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        mapped[i] = uniform_buffers[i].map_memory(&base.device, buffer_size) as _;
    }

    (uniform_buffers, mapped)
}
