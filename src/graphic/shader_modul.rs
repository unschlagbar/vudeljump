use ash::vk;


pub fn create_shader_modul(device: &ash::Device, raw_code: &[u8]) -> vk::ShaderModule {

    let create_info = vk::ShaderModuleCreateInfo {
        code_size: raw_code.len(),
        p_code: raw_code.as_ptr() as *const u32,
        ..Default::default()
    };

    unsafe { device.create_shader_module(&create_info, None).unwrap() }
}