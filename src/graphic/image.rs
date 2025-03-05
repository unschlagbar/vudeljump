#![allow(unused)]

use ash::vk;

pub struct Image {
    texture_image: vk::Image,
    texture_image_mem: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
}

impl Image {
    pub fn new() {
        //let ll = vk::ImageView::null();
    }
}