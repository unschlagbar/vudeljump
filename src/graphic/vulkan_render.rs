#![allow(dead_code)]

use std::{ptr, cell::RefCell, ffi::c_void, mem::size_of, ptr::null, rc::Rc, thread::sleep, time::{Duration, Instant}};
use ash::vk::{self, AccessFlags, AttachmentDescriptionFlags, BorderColor, CommandPoolCreateFlags, CompareOp, DescriptorImageInfo, DescriptorType, Extent3D, Filter, Format, Framebuffer, ImageLayout, ImageTiling, ImageUsageFlags, ImageView, MemoryPropertyFlags, PipelineStageFlags, SampleCountFlags, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode, ShaderStageFlags};
use cgmath::{ortho, Point3, SquareMatrix, Vector3};
use cgmath::Matrix4;
use iron_oxide::{graphics::{self, SinlgeTimeCommands, VkBase}, primitives::Vec2, ui::UiState};
use winit::{dpi::PhysicalSize, raw_window_handle::HasDisplayHandle, window::Window};

use crate::game::World;

use super::{buffer::create_uniform_buffers, uniform_buffer_object::UniformBufferObject, InstanceData};
use super::main_pipeline;

pub const MAXFRAMESINFLIGHT: usize = 2;

pub struct VulkanRender {
    pub window: Window,
    pub window_size: PhysicalSize<u32>,

    pub base: iron_oxide::graphics::VkBase,

    pub swapchain: super::Swapchain,
    pub render_pass: vk::RenderPass,

    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    pub command_pool: vk::CommandPool,
    pub single_time_command_pool: vk::CommandPool,

    pub vertex_count: u32,

    pub instance_count: u32,
    pub instance_buffer: graphics::Buffer,

    uniform_buffers: [graphics::Buffer; MAXFRAMESINFLIGHT],
    uniform_buffers_mapped: [*mut c_void; MAXFRAMESINFLIGHT],

    ui_uniform_buffers: [graphics::Buffer; MAXFRAMESINFLIGHT],
    ui_uniform_buffers_mapped: [*mut c_void; MAXFRAMESINFLIGHT],

    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    ui_descriptor_pool: vk::DescriptorPool,
    pub ui_descriptor_sets: Vec<vk::DescriptorSet>,
    pub ui_descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,

    pub command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: [vk::Semaphore; MAXFRAMESINFLIGHT],
    render_finsih_semaphores: [vk::Semaphore; MAXFRAMESINFLIGHT],
    in_flight_fences: [vk::Fence; MAXFRAMESINFLIGHT],
    pub world_fences: [vk::Fence; MAXFRAMESINFLIGHT],
    ui_fences: [vk::Fence; MAXFRAMESINFLIGHT],
    pub current_frame: usize,

    pub main_framebuffer: Vec<Framebuffer>,

    texture_image: graphics::Image,
    pub texture_sampler: vk::Sampler,

    font_atlas: graphics::Image,

    pub depth_image: graphics::Image,

    start_time: Duration,

    pub ui_state: Rc<RefCell<UiState>>,
    world: *const World,
    pub renderer: u8,
}

impl VulkanRender {
    pub fn create(window: Window, world: &World) -> Self {
        let start_time = Instant::now();

        let base = VkBase::create(unsafe { ash_window::enumerate_required_extensions(window.display_handle().unwrap_unchecked().as_raw()).unwrap_unchecked()}.to_vec() as _, &window, 0);

        let rt_usage = vk::BufferUsageFlags::empty();

        let command_pool = Self::create_command_pool(&base);
        let single_time_command_pool = Self::create_single_time_command_pool(&base);

        let window_size = window.inner_size();
        let cmd_buf = SinlgeTimeCommands::begin(&base, &single_time_command_pool);
        let depth_image = Self::create_depth_resources(&base, &cmd_buf, Extent3D { width: window_size.width, height: window_size.height, depth: 1 });
        SinlgeTimeCommands::end(&base, &single_time_command_pool, cmd_buf);

        let swapchain = super::Swapchain::create(&base, window_size);

        let render_pass = Self::create_render_pass(&base, swapchain.format, true, true, false, true);

        let descriptor_set_layout = create_descriptor_set_layout(&base.device);
        let ui_descriptor_set_layout = create_ui_descriptor_set_layout(&base.device);
        let (pipeline_layout, pipeline) = main_pipeline::create_main_pipeline(&base.device, window_size, render_pass, &descriptor_set_layout);
        let mut texture_image = Self::create_texture_image(&base, &single_time_command_pool);
        let mut font_atlas = Self::create_font_atlas(&base, &single_time_command_pool);
        let texture_sampler = Self::create_texture_sampler(&base.device);

        let instances = world.get_instances();

        let vertex_count = 4;

        //let ui_instances;
        //{
        //    let mut mut_ui = world.ui.borrow_mut();
        //    mut_ui.build(Vec2::new(window_size.width as f32, window_size.height as f32));
        //    ui_instances = mut_ui.get_instaces(Vec2::new(window_size.width as f32, window_size.height as f32));
        //}

        let ui_state = world.ui.clone();

        let instance_buffer = graphics::Buffer::device_local(&base, &single_time_command_pool, size_of::<Matrix4<f32>>() as u64, instances.len() as u64, instances.as_ptr() as _, vk::BufferUsageFlags::VERTEX_BUFFER | rt_usage);

        texture_image.create_view(&base, vk::ImageAspectFlags::COLOR);
        font_atlas.create_view(&base, vk::ImageAspectFlags::COLOR);
        let (uniform_buffers, uniform_buffers_mapped) = create_uniform_buffers(&base);
        let (ui_uniform_buffers, ui_uniform_buffers_mapped) = create_uniform_buffers(&base);
        
        let descriptor_pool = create_descriptor_pool(&base.device);
        let ui_descriptor_pool = create_ui_descriptor_pool(&base.device);
        let descriptor_sets = create_descriptor_sets(&base.device, &descriptor_pool, &descriptor_set_layout, &uniform_buffers, texture_sampler, texture_image.view, size_of::<UniformBufferObject>() as _);
        let ui_descriptor_sets = create_ui_descriptor_sets(&base.device, &ui_descriptor_pool, &ui_descriptor_set_layout, &ui_uniform_buffers, texture_sampler, &[font_atlas.view, texture_image.view], size_of::<UniformBufferObject>() as _);
        //unsafe { base.device.destroy_descriptor_set_layout(ui_descriptor_set_layout, None) };
        //unsafe { device.destroy_descriptor_set_layout(descriptor_set_layout, None) };
        
        let command_buffers = Self::create_command_buffers(&base.device, &command_pool);
        let main_framebuffer = Self::create_framebuffers(&base.device, &swapchain.image_views, &depth_image.view, &render_pass, window_size);
        let (image_available_semaphores, render_finsih_semaphores, in_flight_fences, world_fences, ui_fences)= Self::create_sync_object(&base.device);

        Self::init_ui_uniform_buffer(window_size, &ui_uniform_buffers_mapped);

        let world = world as *const World;

        println!("{:?}", start_time.elapsed());

        Self {
            window,
            window_size,
            base,
            swapchain,
            pipeline_layout,
            render_pass,
            graphics_pipeline: pipeline,
            main_framebuffer,
            command_pool,
            single_time_command_pool,
    
            vertex_count,

            instance_count: instances.len() as _,
            instance_buffer,
    
            uniform_buffers,
            uniform_buffers_mapped,
            ui_uniform_buffers,
            ui_uniform_buffers_mapped,
    
            descriptor_pool,
            ui_descriptor_pool,
            descriptor_sets,
            ui_descriptor_sets,
            ui_descriptor_set_layout,
            descriptor_set_layout,
    
            command_buffers,
            image_available_semaphores,
            render_finsih_semaphores,
            in_flight_fences,
            ui_fences,
            world_fences,
    
            current_frame: 0,
            texture_image,
    
            font_atlas,
    
            texture_sampler,
            depth_image,
    
            start_time: Duration::new(0, 0),
            ui_state,
            world,
            renderer: 0,
        }
    }

    pub fn recreate_swapchain(&mut self, new_size: PhysicalSize<u32>) {

        self.window_size = new_size;

        #[cfg(not(target_os = "android"))]
        if new_size.width == 0 || new_size.height == 0 {
            sleep(Duration::from_millis(100));
            return;
        }

        unsafe { self.base.device.device_wait_idle().unwrap_unchecked() };
        unsafe { self.swapchain.destroy(&self.base.device, &self.main_framebuffer) };
        self.depth_image.destroy(&self.base.device);

        let cmd_buf = SinlgeTimeCommands::begin(&self.base, &self.single_time_command_pool);
        self.depth_image = Self::create_depth_resources(&self.base, &cmd_buf, Extent3D { width: self.window_size.width, height: self.window_size.height, depth: 1 });
        SinlgeTimeCommands::end(&self.base, &self.single_time_command_pool, cmd_buf);

        self.swapchain.recreate(&self.base, new_size);
        self.main_framebuffer = Self::create_framebuffers(&self.base.device, &self.swapchain.image_views, &self.depth_image.view, &self.render_pass, new_size);
        self.update_ui_uniform_buffer();
    }

    fn create_render_pass(base: &VkBase, format: vk::SurfaceFormatKHR, clear: bool, depth: bool, has_previus: bool, is_final: bool) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription {
            format: format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: if clear {vk::AttachmentLoadOp::CLEAR} else { vk::AttachmentLoadOp::DONT_CARE },
            store_op: if is_final {vk::AttachmentStoreOp::STORE} else {vk::AttachmentStoreOp::STORE},
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: if has_previus {vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL} else { vk::ImageLayout::UNDEFINED },
            final_layout: if is_final {vk::ImageLayout::PRESENT_SRC_KHR} else { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL },
            flags: AttachmentDescriptionFlags::empty()
        };

        let depth_attachment = vk::AttachmentDescription {
            format: Format::D24_UNORM_S8_UINT,
            samples: SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            final_layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            flags: AttachmentDescriptionFlags::empty(),
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let attachments;
        if depth {
            attachments = vec![color_attachment, depth_attachment];
        } else {
            attachments = vec![color_attachment];
        }

        let subpasses = [
            vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                color_attachment_count: 1,
                p_color_attachments: &color_attachment_ref as _,
                p_depth_stencil_attachment: if depth {&depth_attachment_ref} else {null()},
                ..Default::default()
            },
            vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                color_attachment_count: 1,
                p_color_attachments: &color_attachment_ref as _,
                p_depth_stencil_attachment: if depth {&depth_attachment_ref} else {null()},
                ..Default::default()
            },
        ];

        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: vk::AccessFlags::empty(),
                dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_access_mask: if depth {AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE} else {AccessFlags::COLOR_ATTACHMENT_WRITE},
                dependency_flags: vk::DependencyFlags::default(),
            },
            vk::SubpassDependency {
                src_subpass: 0,
                dst_subpass: 1,
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: if depth {AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE} else {AccessFlags::COLOR_ATTACHMENT_WRITE},
                dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_access_mask: if depth {AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_WRITE} else {AccessFlags::COLOR_ATTACHMENT_WRITE},
                dependency_flags: vk::DependencyFlags::default(),
            }
        ];

        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as _,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as _,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as _,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        unsafe { base.device.create_render_pass(&render_pass_info, None).unwrap() }

    }

    fn create_framebuffers(device: &ash::Device, image_views: &Vec<vk::ImageView>, depth_image_view: &vk::ImageView, render_pass: &vk::RenderPass, window_size: winit::dpi::PhysicalSize<u32>) -> Vec<vk::Framebuffer> {
        let mut swapchain_framebuffers = Vec::with_capacity(image_views.len());
        for image_view in image_views {

            let attachments = [*image_view, *depth_image_view];
            let main_create_info = vk::FramebufferCreateInfo {
                render_pass: *render_pass,
                attachment_count: attachments.len() as _,
                p_attachments: attachments.as_ptr(),
                width: window_size.width,
                height: window_size.height,
                layers: 1,
                ..Default::default()
            };

            swapchain_framebuffers.push(unsafe { device.create_framebuffer(&main_create_info, None).unwrap() });
        }
        swapchain_framebuffers
    }

    fn create_command_pool(base: &VkBase) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&pool_info, None).unwrap() }
    }

    fn create_single_time_command_pool(base: &VkBase) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: base.queue_family_index,
            ..Default::default()
        };

        unsafe { base.device.create_command_pool(&pool_info, None).unwrap() }
    }

    fn create_command_buffers(device: &ash::Device, command_pool: &vk::CommandPool) -> Vec<vk::CommandBuffer> {
        let aloc_info = vk::CommandBufferAllocateInfo {
            command_pool: *command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAXFRAMESINFLIGHT as _,
            ..Default::default()
        };

        unsafe { device.allocate_command_buffers(&aloc_info).unwrap() }
    }

    fn record_command_buffer(&mut self, index: u32) {

        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];

        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: self.main_framebuffer[index as usize],
            render_area: vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: self.window_size.width, height: self.window_size.height }},
            clear_value_count: clear_values.len() as _,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        let view_port = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.window_size.width as f32,
            height: self.window_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0
        };
        
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: self.window_size.width, height: self.window_size.height },
        };

        let device = &self.base.device;
        
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        
        unsafe {
            device.begin_command_buffer(self.command_buffers[self.current_frame], &begin_info).unwrap();
            
            device.cmd_set_scissor(self.command_buffers[self.current_frame], 0, &[scissor]);
            device.cmd_set_viewport(self.command_buffers[self.current_frame], 0, &[view_port]);
            
            if self.renderer != 0 {
            } else {
                device.cmd_begin_render_pass(self.command_buffers[self.current_frame], &render_pass_info, vk::SubpassContents::INLINE);
                device.cmd_bind_pipeline(self.command_buffers[self.current_frame], vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);
                device.cmd_bind_vertex_buffers(self.command_buffers[self.current_frame], 0, &[self.instance_buffer.inner], &[0]);
                device.cmd_bind_descriptor_sets(self.command_buffers[self.current_frame], vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[self.descriptor_sets[self.current_frame]], &[]);
                device.cmd_draw(self.command_buffers[self.current_frame], 4, self.instance_count, 0, 0);
            }

            device.cmd_next_subpass(self.command_buffers[self.current_frame], vk::SubpassContents::INLINE);
            self.ui_state.borrow().draw(&self.base.device, self.command_buffers[self.current_frame], &self.ui_descriptor_sets[self.current_frame]);
            device.cmd_end_render_pass(self.command_buffers[self.current_frame]);
            
            device.end_command_buffer(self.command_buffers[self.current_frame]).unwrap();
        };
    }

    fn create_sync_object(device: &ash::Device) -> ([vk::Semaphore; MAXFRAMESINFLIGHT], [vk::Semaphore; MAXFRAMESINFLIGHT], [vk::Fence; MAXFRAMESINFLIGHT], [vk::Fence; MAXFRAMESINFLIGHT], [vk::Fence; MAXFRAMESINFLIGHT]) {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let fence_info2 = vk::FenceCreateInfo {
            ..Default::default()
        };

        let mut image_available_semaphores = [vk::Semaphore::null(); MAXFRAMESINFLIGHT];
        let mut render_finsih_semaphores = [vk::Semaphore::null(); MAXFRAMESINFLIGHT];
        let mut in_flight_fences = [vk::Fence::null(); MAXFRAMESINFLIGHT];
        let mut world_fences = [vk::Fence::null(); MAXFRAMESINFLIGHT];
        let mut ui_fences = [vk::Fence::null(); MAXFRAMESINFLIGHT];

        for i in 0..MAXFRAMESINFLIGHT {
            unsafe {
                image_available_semaphores[i] = device.create_semaphore(&semaphore_info, None).unwrap_unchecked();
                render_finsih_semaphores[i] = device.create_semaphore(&semaphore_info, None).unwrap_unchecked();
                in_flight_fences[i] = device.create_fence(&fence_info, None).unwrap_unchecked();
                world_fences[i] = device.create_fence(&fence_info2, None).unwrap_unchecked();
                ui_fences[i] = device.create_fence(&fence_info2, None).unwrap_unchecked();
            }
        }

        (image_available_semaphores, render_finsih_semaphores, in_flight_fences, world_fences, ui_fences)

    }

    pub fn draw_frame(&mut self) {
        let time = Instant::now();

        let window_size = self.window.inner_size();

        if window_size.width == 0 || window_size.height == 0 {
            sleep(Duration::from_millis(25));
            return;
        }

        unsafe {
            self.base.device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX).unwrap();
            self.base.device.reset_fences(&[self.in_flight_fences[self.current_frame]]).unwrap();
            self.base.device.reset_command_buffer(self.command_buffers[self.current_frame], vk::CommandBufferResetFlags::empty()).unwrap()
        };

        let image_index = unsafe { 
            let result = self.swapchain.loader.acquire_next_image(self.swapchain.swapchain, u64::MAX, self.image_available_semaphores[self.current_frame], vk::Fence::null());
            match result {
                Ok(result) => {
                    if result.1 {
                        return;
                    }
                    result.0
                }, 
                Err(_) => return
            }
        };

        if self.ui_state.borrow().dirty {
            self.upload_ui();
        }
        
        let instances = self.world().get_instances();
        self.instance_count = instances.len() as u32;
        self.instance_buffer.update(&self.base, &self.command_pool, size_of::<InstanceData>() as u64, instances.len() as u64, instances.as_ptr() as _, vk::BufferUsageFlags::VERTEX_BUFFER);

        self.record_command_buffer(image_index);
        self.update_uniform_buffer();


        let submit_info = vk::SubmitInfo {
            p_wait_semaphores: &self.image_available_semaphores[self.current_frame],
            wait_semaphore_count: 1,
            p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[self.current_frame],
            signal_semaphore_count: 1,
            p_signal_semaphores: &self.render_finsih_semaphores[self.current_frame],
            ..Default::default()
        };

        unsafe { self.base.device.queue_submit(self.base.queue, &[submit_info], self.in_flight_fences[self.current_frame]).unwrap() };

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.render_finsih_semaphores[self.current_frame],
            swapchain_count: 1,
            p_swapchains: &self.swapchain.swapchain,
            p_image_indices: &image_index,
            ..Default::default()
        };

        if unsafe { self.swapchain.loader.queue_present(self.base.queue, &present_info).is_err() } {
            self.recreate_swapchain(self.window.inner_size());
            return;
        }

        self.current_frame = (self.current_frame + 1) % MAXFRAMESINFLIGHT;
        self.start_time += time.elapsed();
    }

    #[inline]
    fn update_uniform_buffer(&mut self) {

        let aspect = 1.0 - (self.window_size.width as f32 - 400.0) / self.window_size.width as f32;

        let bottom_view = self.world().current_view;

        let ubo = UniformBufferObject {
            view: Matrix4::look_at_rh(Point3::new(0.0, -(self.window_size.height as f32) * aspect - bottom_view, 1.0), Point3::new(0.0, -(self.window_size.height as f32) * aspect - bottom_view, 0.0), Vector3::unit_y()),
            proj: ortho(0.0, self.window_size.width.min(400) as f32, 0.0, self.window_size.height as f32 * aspect, -100.0, 100.0),
        };

        for uniform_buffer in self.uniform_buffers_mapped {

            unsafe { ptr::copy_nonoverlapping(&ubo as *const UniformBufferObject, uniform_buffer as _, 1) };
        }
    }

    fn update_ui_uniform_buffer(&mut self) {

        let ubo = UniformBufferObject {
            view: Matrix4::identity(),
            proj: ortho(0.0, self.window_size.width as _, 0.0, self.window_size.height as _, -100.0, 100.0),
        };
        for i in 0..MAXFRAMESINFLIGHT {
            unsafe { ptr::copy_nonoverlapping(&ubo as *const UniformBufferObject, self.ui_uniform_buffers_mapped[i] as _, 1) };
        }
    }

    fn init_ui_uniform_buffer(window_size: winit::dpi::PhysicalSize<u32>, uni_mapped: &[*mut c_void; MAXFRAMESINFLIGHT]) {

        let ubo = UniformBufferObject {
            view: Matrix4::identity(),
            proj: ortho(0.0, window_size.width as _, 0.0, window_size.height as _, -100.0, 100.0),
        };
        for i in 0..MAXFRAMESINFLIGHT {
            unsafe { ptr::copy_nonoverlapping(&ubo as *const UniformBufferObject, uni_mapped[i] as _, 1) };
        }
    }

    fn create_texture_image(base: &VkBase, command_pool: &vk::CommandPool) -> graphics::Image {
        let decoder = png::Decoder::new(&include_bytes!("C:/Dev/vudeljump/textures/texture.png")[..]);

        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let width = info.width;
        let height = info.height;
        let image_size = buf.len() as u64;
        let extent = Extent3D { width, height, depth: 1 };
        
        let staging_buffer = graphics::Buffer::create(base, image_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let mapped_memory = staging_buffer.map_memory(&base.device, image_size);
        unsafe { 
            ptr::copy_nonoverlapping(buf.as_ptr(), mapped_memory as _, image_size as usize);
        };
        staging_buffer.unmap_memory(&base.device);

        //let (texture_image, textures_image_memory) = Self::create_image(base, extent, Format::R8G8B8A8_SRGB, ImageTiling::OPTIMAL, ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED, MemoryPropertyFlags::DEVICE_LOCAL);
        let mut texture_image = graphics::Image::create(base, extent, Format::R8G8B8A8_SRGB, ImageTiling::OPTIMAL, ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED, MemoryPropertyFlags::DEVICE_LOCAL);
        let cmd_buf = SinlgeTimeCommands::begin(base, command_pool);
        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        texture_image.copy_from_buffer(base, cmd_buf, &staging_buffer, extent, vk::ImageAspectFlags::COLOR);
        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        SinlgeTimeCommands::end(base, command_pool, cmd_buf);

        staging_buffer.destroy(&base.device);

        texture_image
    }

    fn create_font_atlas(base: &VkBase, command_pool: &vk::CommandPool) -> graphics::Image {
        let decoder = png::Decoder::new(&include_bytes!("C:/Dev/vudeljump/font/default8.png")[..]);

        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let width = info.width;
        let height = info.height;
        let image_size = height as u64 * width as u64;
        let extent = Extent3D { width, height, depth: 1 };
        
        let staging_buffer = graphics::Buffer::create(base, image_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        let mapped_memory = staging_buffer.map_memory(&base.device, image_size);
        unsafe { 
            ptr::copy_nonoverlapping(buf.as_ptr(), mapped_memory as _, image_size as usize);
        };
        staging_buffer.unmap_memory(&base.device);

        let mut texture_image = graphics::Image::create(base, extent, Format::R8_UNORM, ImageTiling::OPTIMAL, ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED, MemoryPropertyFlags::DEVICE_LOCAL);
        
        let cmd_buf = SinlgeTimeCommands::begin(base, command_pool);

        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        texture_image.copy_from_buffer(base, cmd_buf, &staging_buffer, extent, vk::ImageAspectFlags::COLOR);
        texture_image.trasition_layout(base, cmd_buf, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        SinlgeTimeCommands::end(base, command_pool, cmd_buf);

        staging_buffer.destroy(&base.device);

        texture_image
    }

    fn create_texture_sampler(device: &ash::Device) -> Sampler {
        let create_info = SamplerCreateInfo {
            mag_filter: Filter::NEAREST,
            min_filter: Filter::NEAREST,
            mipmap_mode: SamplerMipmapMode::NEAREST,
            address_mode_u: SamplerAddressMode::REPEAT,
            address_mode_v: SamplerAddressMode::REPEAT,
            address_mode_w: SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::FALSE,
            max_anisotropy: 0.0,
            compare_enable: vk::FALSE,
            compare_op: CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: vk::LOD_CLAMP_NONE,
            border_color: BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            ..Default::default()
        };

        unsafe { device.create_sampler(&create_info, None).unwrap() }
    }

    fn create_depth_resources(base: &VkBase, cmd_buf: &vk::CommandBuffer, extent: Extent3D) -> graphics::Image {
        let mut depth_image = graphics::Image::create(base, extent, Format::D24_UNORM_S8_UINT, ImageTiling::OPTIMAL, ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, MemoryPropertyFlags::DEVICE_LOCAL);
        depth_image.trasition_layout(base, *cmd_buf, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        depth_image.create_view(base, vk::ImageAspectFlags::DEPTH);
        depth_image
    }

    pub fn update_ui(&mut self, new_size: PhysicalSize<u32>) {
        self.ui_state.borrow_mut().update(&self.base, Vec2::new(new_size.width as f32, new_size.height as f32), &self.single_time_command_pool);
    }

    pub fn upload_ui(&mut self) {
        self.ui_state.borrow_mut().upload(&self.base, Vec2::new(self.window_size.width as f32, self.window_size.height as f32), &self.single_time_command_pool);
    }

    const fn world(&self) -> &World {
        unsafe { &*self.world }
    }


    pub fn destroy(&mut self) {
        let device = &self.base.device;
        #[cfg(debug_assertions)]
        unsafe { self.base.debug_utils.destroy_debug_utils_messenger(self.base.utils_messenger, None) };
        for i in 0..MAXFRAMESINFLIGHT {
            unsafe {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_semaphore(self.render_finsih_semaphores[i], None);
                device.destroy_fence(self.in_flight_fences[i], None);
                device.destroy_fence(self.world_fences[i], None);
                device.destroy_fence(self.ui_fences[i], None);
                self.uniform_buffers[i].destroy(device);
                self.ui_uniform_buffers[i].destroy(device);
            }
        }
        unsafe {
            self.ui_state.borrow().destroy(device);
            device.destroy_command_pool(self.command_pool, None);
            device.destroy_command_pool(self.single_time_command_pool, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_pool(self.ui_descriptor_pool, None);
            device.destroy_render_pass(self.render_pass, None);
            self.swapchain.destroy(device, &self.main_framebuffer);
            device.destroy_sampler(self.texture_sampler, None);
            self.depth_image.destroy(device);
            self.texture_image.destroy(device);
            self.font_atlas.destroy(device);
            self.base.surface_loader.destroy_surface(self.base.surface, None);
            device.destroy_device(None);
            self.base.instance.destroy_instance(None);
        };
    }

}

impl Drop for VulkanRender {
    fn drop(&mut self) {
        self.destroy();
    }
}

fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {

    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: null(),
        _marker: std::marker::PhantomData,
    };

    let sampler_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_count: 1,
        descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
        stage_flags: ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: null(),
        _marker: std::marker::PhantomData,
    };

    let bindings = [ubo_layout_binding, sampler_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as _,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };

    unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
}

fn create_ui_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {

    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: null(),
        _marker: std::marker::PhantomData,
    };

    let sampler_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 2,
        stage_flags: ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: null(),
        _marker: std::marker::PhantomData,
    };

    let bindings = [ubo_layout_binding, sampler_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as _,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };

    unsafe { device.create_descriptor_set_layout(&layout_info, None).unwrap() }
}

fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {

    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAXFRAMESINFLIGHT as _,
        },
        vk::DescriptorPoolSize {
            ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAXFRAMESINFLIGHT as _,
        }
    ];

    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: pool_sizes.len() as _,
        p_pool_sizes: pool_sizes.as_ptr(),
        max_sets: MAXFRAMESINFLIGHT as _,
        ..Default::default()
    };

    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
}

fn create_ui_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {

    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAXFRAMESINFLIGHT as _,
        },
        vk::DescriptorPoolSize {
            ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAXFRAMESINFLIGHT as u32 * 2,
        }
    ];

    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: pool_sizes.len() as _,
        p_pool_sizes: pool_sizes.as_ptr(),
        max_sets: MAXFRAMESINFLIGHT as _,
        ..Default::default()
    };

    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
}

fn create_descriptor_sets(device: &ash::Device, descriptor_pool: &vk::DescriptorPool, descriptor_set_layout: &vk::DescriptorSetLayout, uniform_buffers: &[graphics::Buffer], textures_sampler: Sampler, texture_image_view: ImageView, ubo_size: u64) -> Vec<vk::DescriptorSet> {

    let layouts: [vk::DescriptorSetLayout; MAXFRAMESINFLIGHT] = [*descriptor_set_layout; MAXFRAMESINFLIGHT];

    let allocate_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool: *descriptor_pool,
        descriptor_set_count: MAXFRAMESINFLIGHT as _,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info).unwrap() };

    for i in 0..MAXFRAMESINFLIGHT {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: uniform_buffers[i].inner,
            offset: 0,
            range: ubo_size,
        };

        let image_info = DescriptorImageInfo {
            sampler: textures_sampler,
            image_view: texture_image_view,
            image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        let descriptor_writes = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                p_image_info: &image_info,
                ..Default::default()
            }
        ];

        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    }

    descriptor_sets
}

fn create_ui_descriptor_sets(device: &ash::Device, descriptor_pool: &vk::DescriptorPool, descriptor_set_layout: &vk::DescriptorSetLayout, uniform_buffers: &[graphics::Buffer], textures_sampler: Sampler, texture_image_views: &[ImageView], ubo_size: u64) -> Vec<vk::DescriptorSet> {

    let layouts: [vk::DescriptorSetLayout; MAXFRAMESINFLIGHT] = [*descriptor_set_layout; MAXFRAMESINFLIGHT];

    let allocate_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool: *descriptor_pool,
        descriptor_set_count: MAXFRAMESINFLIGHT as _,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info).unwrap() };

    for i in 0..MAXFRAMESINFLIGHT {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer: uniform_buffers[i].inner,
            offset: 0,
            range: ubo_size,
        };

        let mut image_infos = Vec::with_capacity(texture_image_views.len());

        for image_view in texture_image_views {
            image_infos.push(
                DescriptorImageInfo {
                    sampler: textures_sampler,
                    image_view: *image_view,
                    image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }
            );
        }

        let descriptor_writes = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 1,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: image_infos.len() as _,
                p_image_info: image_infos.as_ptr(),
                ..Default::default()
            }
        ];

        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    }

    descriptor_sets
}

#[test] 
fn test() {
    use iron_oxide::ui::Font;
    let f = Font::parse("C:/Dev/raytracing/font/std.fef".into());
    println!("{}", b' ');
    let data = f.get_data(b'!');
    println!("{},{},{},{}", data.0, data.1, data.2, data.3);
}