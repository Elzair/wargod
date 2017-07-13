use std::cmp;
use dacite::core as dc;
use dacite::khr_swapchain as ds;
use dacite::khr_surface::{COMPOSITE_ALPHA_OPAQUE_BIT_KHR, ColorSpaceKhr, PresentModeKhr};
use super::device;

pub struct Internal {
    pub framebuffers: Vec<dc::Framebuffer>,
    pub render_pass: dc::RenderPass,
    pub format: dc::Format,
    pub image_views: Vec<dc::ImageView>,
    pub extent: dc::Extent2D,
    pub swapchain: ds::SwapchainKhr,
}

impl Internal {
    pub fn new(
        device: &device::Internal,
        preferred_extent: &dc::Extent2D
    ) -> Result<Self, ()> {
        let ( format,
              image_views,
              extent,
              swapchain ) = create_swapchain(device, preferred_extent)?;
        
        let render_pass = create_render_pass(&device.device, format.clone())?;

        let framebuffers = create_framebuffers(&device.device, &image_views, extent.clone(), &render_pass)?;

        Ok(Internal {
            framebuffers: framebuffers,
            render_pass: render_pass,
            format: format,
            image_views: image_views,
            extent: extent,
            swapchain: swapchain,
        })
    }
}

fn create_swapchain(
    device: &device::Internal,
    preferred_extent: &dc::Extent2D
) -> Result<(dc::Format, Vec<dc::ImageView>, dc::Extent2D, ds::SwapchainKhr), ()> {
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
        if (surface_format.format == dc::Format::B8G8R8A8_UNorm) &&
            (surface_format.color_space == ColorSpaceKhr::SRGBNonLinear) {
                format = Some(surface_format.format);
                color_space = Some(surface_format.color_space);
                break;
            }
    }

    let format = format.ok_or_else(|| {
        println!("No suitable surface format found");
    })?;

    let (image_sharing_mode, queue_family_indices) = if device.queue_family_indices.graphics == device.queue_family_indices.present {
        (dc::SharingMode::Exclusive, vec![])
    }
    else {
        (dc::SharingMode::Concurrent, vec![device.queue_family_indices.graphics, device.queue_family_indices.present])
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
        if mode == PresentModeKhr::Fifo {
            present_mode = Some(PresentModeKhr::Fifo);
            break;
        }
        else if mode == PresentModeKhr::Immediate {
            present_mode = Some(PresentModeKhr::Immediate);
        }
    }

    if present_mode.is_none() {
        println!("No suitable present mode found");
        return Err(());
    }

    let create_info = ds::SwapchainCreateInfoKhr {
        flags: ds::SwapchainCreateFlagsKhr::empty(),
        surface: device.surface.clone(),
        min_image_count: min_image_count,
        image_format: format,
        image_color_space: color_space.unwrap(),
        image_extent: extent,
        image_array_layers: 1,
        image_usage: dc::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        image_sharing_mode: image_sharing_mode,
        queue_family_indices: queue_family_indices,
        pre_transform: capabilities.current_transform,
        composite_alpha: COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
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
        let create_info = dc::ImageViewCreateInfo {
            flags: dc::ImageViewCreateFlags::empty(),
            image: image.clone(),
            view_type: dc::ImageViewType::Type2D,
            format: format,
            components: dc::ComponentMapping::identity(),
            subresource_range: dc::ImageSubresourceRange {
                aspect_mask: dc::IMAGE_ASPECT_COLOR_BIT,
                base_mip_level: 0,
                level_count: dc::OptionalMipLevels::MipLevels(1),
                base_array_layer: 0,
                layer_count: dc::OptionalArrayLayers::ArrayLayers(1),
            },
            chain: None,
        };

        let image_view = device.device.create_image_view(&create_info, None).map_err(|e| {
            println!("Failed to create swapchain image view ({})", e);
        })?;

        image_views.push(image_view);
    }

    Ok((format, image_views, extent, swapchain))
}

fn create_render_pass(
    device: &dc::Device,
    format: dc::Format
) -> Result<dc::RenderPass, ()> {
    let create_info = dc::RenderPassCreateInfo {
        flags: dc::RenderPassCreateFlags::empty(),
        attachments: vec![dc::AttachmentDescription {
            flags: dc::AttachmentDescriptionFlags::empty(),
            format: format,
            samples: dc::SAMPLE_COUNT_1_BIT,
            load_op: dc::AttachmentLoadOp::Clear,
            store_op: dc::AttachmentStoreOp::Store,
            stencil_load_op: dc::AttachmentLoadOp::DontCare,
            stencil_store_op: dc::AttachmentStoreOp::DontCare,
            initial_layout: dc::ImageLayout::Undefined,
            final_layout: dc::ImageLayout::PresentSrcKhr,
        }],
        subpasses: vec![dc::SubpassDescription {
            flags: dc::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: dc::PipelineBindPoint::Graphics,
            input_attachments: vec![],
            color_attachments: vec![dc::AttachmentReference {
                attachment: dc::AttachmentIndex::Index(0),
                layout: dc::ImageLayout::ColorAttachmentOptimal,
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
    device: &dc::Device,
    image_views: &Vec<dc::ImageView>,
    extent: dc::Extent2D,
    render_pass: &dc::RenderPass
) -> Result<Vec<dc::Framebuffer>, ()> {
    let mut framebuffers = Vec::with_capacity(image_views.len());
    for image_view in image_views {
        let create_info = dc::FramebufferCreateInfo {
            flags: dc::FramebufferCreateFlags::empty(),
            render_pass: render_pass.clone(),
            attachments: vec![image_view.clone()],
            width: extent.width,
            height: extent.height,
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
