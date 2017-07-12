extern crate winit;
extern crate dacite;
extern crate dacite_winit;
use std::cmp;
use window;

pub mod device;

struct SwapchainSettings {
    swapchain: dacite::khr_swapchain::SwapchainKhr,
    extent: dacite::core::Extent2D,
    image_views: Vec<dacite::core::ImageView>,
    format: dacite::core::Format,
}

pub struct Core {
    pub device: device::Device,
    pub swapchain: dacite::khr_swapchain::SwapchainKhr,
    pub image_views: Vec<dacite::core::ImageView>,
    pub render_pass: dacite::core::RenderPass,
    pub framebuffers: Vec<dacite::core::Framebuffer>,
}

impl Core {
    pub fn new(window: &window::Window) -> Result<Self, ()> {
        let device = device::Device::new(window)?;
        
        let swapchain_settings = create_swapchain(&device, &window.extent)?;

        let render_pass = create_render_pass(&device.device, &swapchain_settings)?;
        let framebuffers = create_framebuffers(&device.device,
                                               &swapchain_settings,
                                               &render_pass)?;

        Ok(Core {
            device: device,
            swapchain: swapchain_settings.swapchain,
            image_views: swapchain_settings.image_views,
            render_pass: render_pass,
            framebuffers: framebuffers,
        })
    }
}

fn create_swapchain(
    device: &device::Device,
    preferred_extent: &dacite::core::Extent2D
) -> Result<SwapchainSettings, ()> {
    let capabilities = device.physical_device.get_surface_capabilities_khr(&device.surface).map_err(|e| {
        println!("Failed to get surface capabilities ({})", e);
    })?;

    let min_image_count = match capabilities.max_image_count {
        Some(max_image_count) => cmp::max(capabilities.min_image_count,
                                          cmp::min(3, max_image_count)),
        None => cmp::max(capabilities.min_image_count, 3),
    };

    let surface_formats: Vec<_> = device.physical_device.get_surface_formats_khr(&device.surface).map_err(|e| {
        println!("Failed to get surface formats ({})", e);
    })?;

    let mut format = None;
    let mut color_space = None;
    for surface_format in surface_formats {
        if (surface_format.format == dacite::core::Format::B8G8R8A8_UNorm) &&
            (surface_format.color_space == dacite::khr_surface::ColorSpaceKhr::SRGBNonLinear) {
            format = Some(surface_format.format);
            color_space = Some(surface_format.color_space);
            break;
        }
    }

    let format = format.ok_or_else(|| {
        println!("No suitable surface format found");
    })?;

    let (image_sharing_mode, queue_family_indices) = if device.queue_family_indices.graphics == device.queue_family_indices.present {
        (dacite::core::SharingMode::Exclusive, vec![])
    }
    else {
        (dacite::core::SharingMode::Concurrent, vec![device.queue_family_indices.graphics, device.queue_family_indices.present])
    };

    let extent = match capabilities.current_extent {
        Some(extent) => extent,
        None => *preferred_extent,
    };

    let present_modes: Vec<_> = device.physical_device.get_surface_present_modes_khr(&device.surface).map_err(|e| {
        println!("Failed to get surface present modes ({})", e);
    })?;

    let mut present_mode = None;
    for mode in present_modes {
        if mode == dacite::khr_surface::PresentModeKhr::Fifo {
            present_mode = Some(dacite::khr_surface::PresentModeKhr::Fifo);
            break;
        }
        else if mode == dacite::khr_surface::PresentModeKhr::Immediate {
            present_mode = Some(dacite::khr_surface::PresentModeKhr::Immediate);
        }
    }

    if present_mode.is_none() {
        println!("No suitable present mode found");
        return Err(());
    }

    let create_info = dacite::khr_swapchain::SwapchainCreateInfoKhr {
        flags: dacite::khr_swapchain::SwapchainCreateFlagsKhr::empty(),
        surface: device.surface.clone(),
        min_image_count: min_image_count,
        image_format: format,
        image_color_space: color_space.unwrap(),
        image_extent: extent,
        image_array_layers: 1,
        image_usage: dacite::core::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        image_sharing_mode: image_sharing_mode,
        queue_family_indices: queue_family_indices,
        pre_transform: capabilities.current_transform,
        composite_alpha: dacite::khr_surface::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        present_mode: present_mode.unwrap(),
        clipped: true,
        old_swapchain: None,
        chain: None,
    };

    let swapchain = device.device.create_swapchain_khr(&create_info, None).map_err(|e| {
        println!("Failed to create swapchain ({})", e);
    })?;

    let images = swapchain.get_images_khr().map_err(|e| {
        println!("Failed to get swapchain images ({})", e);
    })?;

    let mut image_views = Vec::with_capacity(images.len());
    for image in &images {
        let create_info = dacite::core::ImageViewCreateInfo {
            flags: dacite::core::ImageViewCreateFlags::empty(),
            image: image.clone(),
            view_type: dacite::core::ImageViewType::Type2D,
            format: format,
            components: dacite::core::ComponentMapping::identity(),
            subresource_range: dacite::core::ImageSubresourceRange {
                aspect_mask: dacite::core::IMAGE_ASPECT_COLOR_BIT,
                base_mip_level: 0,
                level_count: dacite::core::OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: dacite::core::OptionalArrayLayers::ArrayLayers(1),
            },
            chain: None,
        };

        let image_view = device.device.create_image_view(&create_info, None).map_err(|e| {
            println!("Failed to create swapchain image view ({})", e);
        })?;

        image_views.push(image_view);
    }

    Ok(SwapchainSettings {
        swapchain: swapchain,
        extent: extent,
        image_views: image_views,
        format: format,
    })
}

fn create_render_pass(
    device: &dacite::core::Device,
    swapchain: &SwapchainSettings
) -> Result<dacite::core::RenderPass, ()> {
    let create_info = dacite::core::RenderPassCreateInfo {
        flags: dacite::core::RenderPassCreateFlags::empty(),
        attachments: vec![dacite::core::AttachmentDescription {
            flags: dacite::core::AttachmentDescriptionFlags::empty(),
            format: swapchain.format,
            samples: dacite::core::SAMPLE_COUNT_1_BIT,
            load_op: dacite::core::AttachmentLoadOp::Clear,
            store_op: dacite::core::AttachmentStoreOp::Store,
            stencil_load_op: dacite::core::AttachmentLoadOp::DontCare,
            stencil_store_op: dacite::core::AttachmentStoreOp::DontCare,
            initial_layout: dacite::core::ImageLayout::Undefined,
            final_layout: dacite::core::ImageLayout::PresentSrcKhr,
        }],
        subpasses: vec![dacite::core::SubpassDescription {
            flags: dacite::core::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: dacite::core::PipelineBindPoint::Graphics,
            input_attachments: vec![],
            color_attachments: vec![dacite::core::AttachmentReference {
                attachment: dacite::core::AttachmentIndex::Index(0),
                layout: dacite::core::ImageLayout::ColorAttachmentOptimal,
            }],
            resolve_attachments: vec![],
            depth_stencil_attachment: None,
            preserve_attachments: vec![],
        }],
        dependencies: vec![],
        chain: None,
    };

    device.create_render_pass(&create_info, None).map_err(|e| {
        println!("Failed to create renderpass ({})", e);
    })
}

fn create_framebuffers(
    device: &dacite::core::Device,
    swapchain: &SwapchainSettings,
    render_pass: &dacite::core::RenderPass
) -> Result<Vec<dacite::core::Framebuffer>, ()> {
    let mut framebuffers = Vec::with_capacity(swapchain.image_views.len());
    for image_view in &swapchain.image_views {
        let create_info = dacite::core::FramebufferCreateInfo {
            flags: dacite::core::FramebufferCreateFlags::empty(),
            render_pass: render_pass.clone(),
            attachments: vec![image_view.clone()],
            width: swapchain.extent.width,
            height: swapchain.extent.height,
            layers: 1,
            chain: None,
        };

        let framebuffer = device.create_framebuffer(&create_info, None).map_err(|e| {
            println!("Failed to create framebuffer ({})", e);
        })?;

        framebuffers.push(framebuffer);
    }

    Ok(framebuffers)
}
