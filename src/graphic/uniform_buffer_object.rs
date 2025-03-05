#![allow(unused)]

use cgmath::Matrix4;

#[repr(align(16))]
pub struct UniformBufferObject {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>
}  