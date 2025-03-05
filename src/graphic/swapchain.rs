use ash::{khr::swapchain, vk::{self, Format, Framebuffer, ImageView, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainKHR}, Device};
use iron_oxide::graphics::VkBase;
use winit::dpi::PhysicalSize;


pub struct Swapchain {
    pub loader: swapchain::Device,
    pub swapchain: SwapchainKHR,
    pub image_views: Vec<ImageView>,
    pub capabilities: SurfaceCapabilitiesKHR,
    pub format: SurfaceFormatKHR,
    pub present_mode: PresentModeKHR,
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
}

impl Swapchain {
    pub fn create(base: &VkBase, window_size: PhysicalSize<u32>) -> Self {
        let loader = swapchain::Device::new(&base.instance, &base.device);
        let (capabilities, format, present_mode) = Self::query_swap_chain_support(base);
        let composite_alpha = if capabilities.supported_composite_alpha.contains(vk::CompositeAlphaFlagsKHR::OPAQUE) {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        } else {
            vk::CompositeAlphaFlagsKHR::INHERIT
        };
        let swapchain = Self::create_swap_chain(window_size, &base.surface, &loader, &capabilities, composite_alpha, format, present_mode, base.queue_family_index);
        let image_views = Self::create_image_views(&loader, &swapchain, &base.device, format.format);

        Self {
            loader,
            swapchain,
            image_views,
            capabilities,
            format,
            present_mode,
            composite_alpha
        }
    }

    pub fn recreate(&mut self, base: &VkBase, window_size: PhysicalSize<u32>) {

        self.capabilities = unsafe { base.surface_loader.get_physical_device_surface_capabilities(base.physical_device, base.surface).unwrap_unchecked() };

        let mut image_count = self.capabilities.min_image_count + 1;
        if self.capabilities.max_image_count > 0 && image_count > self.capabilities.max_image_count {
            image_count = self.capabilities.max_image_count;
        }

        let image_extent = if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            vk::Extent2D { width: window_size.width, height: window_size.height }
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: base.surface,
            min_image_count: image_count,
            image_format: self.format.format,
            image_color_space: self.format.color_space,
            image_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 1,
            p_queue_family_indices: &base.queue_family_index,
            pre_transform: SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: self.composite_alpha,
            present_mode: self.present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        self.swapchain = unsafe { self.loader.create_swapchain(&create_info, None).unwrap_unchecked() };
        self.image_views = Self::create_image_views(&self.loader, &self.swapchain, &base.device, self.format.format);
    }

    fn query_swap_chain_support(base: &VkBase) -> (SurfaceCapabilitiesKHR, vk::SurfaceFormatKHR, vk::PresentModeKHR){
        let capabilities = unsafe { base.surface_loader.get_physical_device_surface_capabilities( base.physical_device, base.surface).unwrap_unchecked() };

        let format: vk::SurfaceFormatKHR = unsafe {
            base.surface_loader.get_physical_device_surface_formats(base.physical_device, base.surface).unwrap_unchecked().into_iter().find(|format| {format.format == vk::Format::R8G8B8A8_SRGB&& format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR}).expect("No suitable format found!")
        };

        let present_mode = unsafe { 
            base.surface_loader.get_physical_device_surface_present_modes(base.physical_device, base.surface).unwrap_unchecked().into_iter().find(|pm| {pm == &vk::PresentModeKHR::FIFO}).expect("No suitable presentmode found!")
        };

        (capabilities, format, present_mode)
    }

    fn create_swap_chain(window_size: PhysicalSize<u32>, surface: &SurfaceKHR, swapchain_loader: &swapchain::Device, capabilities: &SurfaceCapabilitiesKHR, composite_alpha: vk::CompositeAlphaFlagsKHR, format: SurfaceFormatKHR, present_mode: vk::PresentModeKHR, queue_family_index: u32) -> SwapchainKHR {
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let image_extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D { width: window_size.width, height: window_size.height }
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: *surface,
            min_image_count: image_count,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 1,
            p_queue_family_indices: &queue_family_index,
            pre_transform: SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        unsafe { swapchain_loader.create_swapchain(&create_info, None).unwrap_unchecked() }

    }

    fn create_image_views(swapchain_loader: &swapchain::Device, swapchain: &SwapchainKHR, device: &ash::Device, format: Format) -> Vec<vk::ImageView> {
        let present_images = unsafe { swapchain_loader.get_swapchain_images(*swapchain).unwrap() };
        let mut present_image_views = Vec::with_capacity(present_images.len());

        for present_image in present_images {
            let create_info = vk::ImageViewCreateInfo {
                image: present_image,
                view_type: vk::ImageViewType::TYPE_2D,
                format,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
           present_image_views.push(unsafe { device.create_image_view(&create_info, None).unwrap() });
        }

        present_image_views
    }

    pub unsafe fn destroy(&mut self, device: &Device, framebuffer: &Vec<Framebuffer>) {
    
        for i in 0..self.image_views.len() {
            device.destroy_framebuffer(framebuffer[i], None);
        }
    
        for image_view in &self.image_views {
            device.destroy_image_view(*image_view, None);
        }
    
        self.loader.destroy_swapchain(self.swapchain, None);
    }
}
