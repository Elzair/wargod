use winit;
use dacite;
use dacite_winit;
use dacite_winit::WindowExt;
use std::cmp;

pub struct Window {
    pub events_loop: winit::EventsLoop,
    pub window: winit::Window,
}

pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

pub struct DeviceSettings {
    pub physical_device: dacite::core::PhysicalDevice,
    pub queue_family_indices: QueueFamilyIndices,
    pub device_extensions: dacite::core::DeviceExtensions,
}

pub struct SwapchainSettings {
    pub swapchain: dacite::khr_swapchain::SwapchainKhr,
    pub extent: dacite::core::Extent2D,
    pub image_views: Vec<dacite::core::ImageView>,
    pub format: dacite::core::Format,
}

pub fn create_window(extent: &dacite::core::Extent2D) -> Result<Window, ()> {
    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_title("dacite triangle example")
        .with_dimensions(extent.width, extent.height)
        .with_min_dimensions(extent.width, extent.height)
        .with_max_dimensions(extent.width, extent.height)
        .with_visibility(false)
        .build(&events_loop);

    let window = window.map_err(|e| {
        match e {
            winit::CreationError::OsError(e) => println!("Failed to create window ({})", e),
            winit::CreationError::NotSupported => println!("Failed to create window (not supported)"),
        }
    })?;

    Ok(Window {
        events_loop: events_loop,
        window: window,
    })
}

pub fn compute_instance_extensions(window: &winit::Window) -> Result<dacite::core::InstanceExtensions, ()> {
    let available_extensions = dacite::core::Instance::get_instance_extension_properties(None).map_err(|e| {
        println!("Failed to get instance extension properties ({})", e);
    })?;

    let required_extensions = window.get_required_extensions().map_err(|e| match e {
        dacite_winit::Error::Unsupported => println!("The windowing system is not supported"),
        dacite_winit::Error::VulkanError(e) => println!("Failed to get required extensions for the window ({})", e),
    })?;

    let missing_extensions = required_extensions.difference(&available_extensions);
    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    }
    else {
        for (name, spec_version) in missing_extensions.properties() {
            println!("Extension {} (revision {}) missing", name, spec_version);
        }

        Err(())
    }
}

pub fn create_instance(instance_extensions: dacite::core::InstanceExtensions) -> Result<dacite::core::Instance, ()> {
    let application_info = dacite::core::ApplicationInfo {
        application_name: Some("dacite triangle example".to_owned()),
        application_version: 0,
        engine_name: None,
        engine_version: 0,
        api_version: Some(dacite::DACITE_API_VERSION_1_0),
        chain: None,
    };

    let create_info = dacite::core::InstanceCreateInfo {
        flags: dacite::core::InstanceCreateFlags::empty(),
        application_info: Some(application_info),
        enabled_layers: vec![],
        enabled_extensions: instance_extensions,
        chain: None,
    };

    dacite::core::Instance::create(&create_info, None).map_err(|e| {
        println!("Failed to create instance ({})", e);
    })
}

pub fn find_queue_family_indices(physical_device: &dacite::core::PhysicalDevice,
                             surface: &dacite::khr_surface::SurfaceKhr) -> Result<QueueFamilyIndices, ()> {
    let mut graphics = None;
    let mut present = None;

    let queue_family_properties: Vec<_> = physical_device.get_queue_family_properties();
    for (index, queue_family_properties) in queue_family_properties.into_iter().enumerate() {
        if queue_family_properties.queue_count == 0 {
            continue;
        }

        if graphics.is_none() && queue_family_properties.queue_flags.contains(dacite::core::QUEUE_GRAPHICS_BIT) {
            graphics = Some(index);
        }

        if present.is_none() {
            if let Ok(true) = physical_device.get_surface_support_khr(index as u32, surface) {
                present = Some(index);
            }
        }
    }

    if let (Some(graphics), Some(present)) = (graphics, present) {
        Ok(QueueFamilyIndices {
            graphics: graphics as u32,
            present: present as u32,
        })
    }
    else {
        Err(())
    }
}

pub fn check_device_extensions(physical_device: &dacite::core::PhysicalDevice) -> Result<dacite::core::DeviceExtensions, ()> {
    let available_extensions = physical_device.get_device_extension_properties(None).map_err(|e| {
        println!("Failed to get device extension properties ({})", e);
    })?;

    let mut required_extensions = dacite::core::DeviceExtensionsProperties::new();
    required_extensions.add_khr_swapchain(67);

    let missing_extensions = required_extensions.difference(&available_extensions);
    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    }
    else {
        for (name, spec_version) in missing_extensions.properties() {
            println!("Extension {} (revision {}) missing", name, spec_version);
        }

        Err(())
    }
}

pub fn check_device_suitability(physical_device: dacite::core::PhysicalDevice, surface: &dacite::khr_surface::SurfaceKhr) -> Result<DeviceSettings, ()> {
    let queue_family_indices = find_queue_family_indices(&physical_device, surface)?;
    let device_extensions = check_device_extensions(&physical_device)?;

    Ok(DeviceSettings {
        physical_device: physical_device,
        queue_family_indices: queue_family_indices,
        device_extensions: device_extensions,
    })
}

pub fn find_suitable_device(instance: &dacite::core::Instance, surface: &dacite::khr_surface::SurfaceKhr) -> Result<DeviceSettings, ()> {
    let physical_devices = instance.enumerate_physical_devices().map_err(|e| {
        println!("Failed to enumerate physical devices ({})", e);
    })?;

    for physical_device in physical_devices {
        if let Ok(device_settings) = check_device_suitability(physical_device, surface) {
            return Ok(device_settings);
        }
    }

    println!("Failed to find a suitable device");
    Err(())
}

pub fn create_device(physical_device: &dacite::core::PhysicalDevice, device_extensions: dacite::core::DeviceExtensions, queue_family_indices: &QueueFamilyIndices) -> Result<dacite::core::Device, ()> {
    let device_queue_create_infos = vec![
        dacite::core::DeviceQueueCreateInfo {
            flags: dacite::core::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family_indices.graphics,
            queue_priorities: vec![1.0],
            chain: None,
        },
    ];

    let device_create_info = dacite::core::DeviceCreateInfo {
        flags: dacite::core::DeviceCreateFlags::empty(),
        queue_create_infos: device_queue_create_infos,
        enabled_layers: vec![],
        enabled_extensions: device_extensions,
        enabled_features: None,
        chain: None,
    };

    physical_device.create_device(&device_create_info, None).map_err(|e| {
        println!("Failed to create device ({})", e);
    })
}

pub fn create_swapchain(physical_device: &dacite::core::PhysicalDevice, device: &dacite::core::Device, surface: &dacite::khr_surface::SurfaceKhr, preferred_extent: &dacite::core::Extent2D, queue_family_indices: &QueueFamilyIndices) -> Result<SwapchainSettings, ()> {
    let capabilities = physical_device.get_surface_capabilities_khr(surface).map_err(|e| {
        println!("Failed to get surface capabilities ({})", e);
    })?;

    let min_image_count = match capabilities.max_image_count {
        Some(max_image_count) => cmp::max(capabilities.min_image_count, cmp::min(3, max_image_count)),
        None => cmp::max(capabilities.min_image_count, 3),
    };

    let surface_formats: Vec<_> = physical_device.get_surface_formats_khr(surface).map_err(|e| {
        println!("Failed to get surface formats ({})", e);
    })?;

    let mut format = None;
    let mut color_space = None;
    for surface_format in surface_formats {
        if (surface_format.format == dacite::core::Format::B8G8R8A8_UNorm) && (surface_format.color_space == dacite::khr_surface::ColorSpaceKhr::SRGBNonLinear) {
            format = Some(surface_format.format);
            color_space = Some(surface_format.color_space);
            break;
        }
    }

    let format = format.ok_or_else(|| {
        println!("No suitable surface format found");
    })?;

    let (image_sharing_mode, queue_family_indices) = if queue_family_indices.graphics == queue_family_indices.present {
        (dacite::core::SharingMode::Exclusive, vec![])
    }
    else {
        (dacite::core::SharingMode::Concurrent, vec![queue_family_indices.graphics, queue_family_indices.present])
    };

    let extent = match capabilities.current_extent {
        Some(extent) => extent,
        None => *preferred_extent,
    };

    let present_modes: Vec<_> = physical_device.get_surface_present_modes_khr(surface).map_err(|e| {
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
        surface: surface.clone(),
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

    let swapchain = device.create_swapchain_khr(&create_info, None).map_err(|e| {
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

        let image_view = device.create_image_view(&create_info, None).map_err(|e| {
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

pub fn create_render_pass(device: &dacite::core::Device, format: dacite::core::Format) -> Result<dacite::core::RenderPass, ()> {
    let create_info = dacite::core::RenderPassCreateInfo {
        flags: dacite::core::RenderPassCreateFlags::empty(),
        attachments: vec![dacite::core::AttachmentDescription {
            flags: dacite::core::AttachmentDescriptionFlags::empty(),
            format: format,
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

pub fn create_framebuffers(device: &dacite::core::Device, image_views: &[dacite::core::ImageView], render_pass: &dacite::core::RenderPass, extent: &dacite::core::Extent2D) -> Result<Vec<dacite::core::Framebuffer>, ()> {
    let mut framebuffers = Vec::with_capacity(image_views.len());
    for image_view in image_views {
        let create_info = dacite::core::FramebufferCreateInfo {
            flags: dacite::core::FramebufferCreateFlags::empty(),
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

pub fn create_vertex_shader(device: &dacite::core::Device) -> Result<dacite::core::ShaderModule, ()> {
    let vertex_shader_bytes = glsl_vs!{r#"
        #version 450

        out gl_PerVertex {
            vec4 gl_Position;
        };

        layout(location = 0) out vec3 fragColor;

        vec2 positions[3] = vec2[](
            vec2(0.0, -0.5),
            vec2(0.5, 0.5),
            vec2(-0.5, 0.5)
        );

        vec3 colors[3] = vec3[](
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 1.0)
        );

        void main() {
            gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
            fragColor = colors[gl_VertexIndex];
        }
    "#};

    let create_info = dacite::core::ShaderModuleCreateInfo {
        flags: dacite::core::ShaderModuleCreateFlags::empty(),
        code: vertex_shader_bytes.to_vec(),
        chain: None,
    };

    device.create_shader_module(&create_info, None).map_err(|e| {
        println!("Failed to create vertex shader module ({})", e);
    })
}

pub fn create_fragment_shader(device: &dacite::core::Device) -> Result<dacite::core::ShaderModule, ()> {
    let fragment_shader_bytes = glsl_fs!{r#"
        #version 450

        layout(location = 0) in vec3 fragColor;

        layout(location = 0) out vec4 outColor;

        void main() {
            outColor = vec4(fragColor, 1.0);
        }
    "#};

    let create_info = dacite::core::ShaderModuleCreateInfo {
        flags: dacite::core::ShaderModuleCreateFlags::empty(),
        code: fragment_shader_bytes.to_vec(),
        chain: None,
    };

    device.create_shader_module(&create_info, None).map_err(|e| {
        println!("Failed to create fragment shader module ({})", e);
    })
}

pub fn create_pipeline_layout(device: &dacite::core::Device) -> Result<dacite::core::PipelineLayout, ()> {
    let create_info = dacite::core::PipelineLayoutCreateInfo {
        flags: dacite::core::PipelineLayoutCreateFlags::empty(),
        set_layouts: vec![],
        push_constant_ranges: vec![],
        chain: None,
    };

    device.create_pipeline_layout(&create_info, None).map_err(|e| {
        println!("Failed to create pipeline layout ({})", e);
    })
}

pub fn create_pipeline(device: &dacite::core::Device, render_pass: &dacite::core::RenderPass, extent: &dacite::core::Extent2D) -> Result<dacite::core::Pipeline, ()> {
    let vertex_shader = create_vertex_shader(device)?;
    let fragment_shader = create_fragment_shader(device)?;
    let layout = create_pipeline_layout(device)?;

    let create_infos = vec![dacite::core::GraphicsPipelineCreateInfo {
        flags: dacite::core::PipelineCreateFlags::empty(),
        stages: vec![
            dacite::core::PipelineShaderStageCreateInfo {
                flags: dacite::core::PipelineShaderStageCreateFlags::empty(),
                stage: dacite::core::SHADER_STAGE_VERTEX_BIT,
                module: vertex_shader.clone(),
                name: "main".to_owned(),
                specialization_info: None,
                chain: None,
            },
            dacite::core::PipelineShaderStageCreateInfo {
                flags: dacite::core::PipelineShaderStageCreateFlags::empty(),
                stage: dacite::core::SHADER_STAGE_FRAGMENT_BIT,
                module: fragment_shader.clone(),
                name: "main".to_owned(),
                specialization_info: None,
                chain: None,
            },
        ],
        vertex_input_state: dacite::core::PipelineVertexInputStateCreateInfo {
            flags: dacite::core::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_descriptions: vec![],
            vertex_attribute_descriptions: vec![],
            chain: None,
        },
        input_assembly_state: dacite::core::PipelineInputAssemblyStateCreateInfo {
            flags: dacite::core::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: dacite::core::PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            chain: None,
        },
        tessellation_state: None,
        viewport_state: Some(dacite::core::PipelineViewportStateCreateInfo {
            flags: dacite::core::PipelineViewportStateCreateFlags::empty(),
            viewports: vec![dacite::core::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
            scissors: vec![dacite::core::Rect2D::new(dacite::core::Offset2D::zero(), *extent)],
            chain: None,
        }),
        rasterization_state: dacite::core::PipelineRasterizationStateCreateInfo {
            flags: dacite::core::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: dacite::core::PolygonMode::Fill,
            cull_mode: dacite::core::CULL_MODE_NONE,
            front_face: dacite::core::FrontFace::Clockwise,
            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
            chain: None,
        },
        multisample_state: Some(dacite::core::PipelineMultisampleStateCreateInfo {
            flags: dacite::core::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: dacite::core::SAMPLE_COUNT_1_BIT,
            sample_shading_enable: false,
            min_sample_shading: 1.0,
            sample_mask: vec![],
            alpha_to_coverage_enable: false,
            alpha_to_one_enable: false,
            chain: None,
        }),
        depth_stencil_state: None,
        color_blend_state: Some(dacite::core::PipelineColorBlendStateCreateInfo {
            flags: dacite::core::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: false,
            logic_op: dacite::core::LogicOp::Copy,
            attachments: vec![dacite::core::PipelineColorBlendAttachmentState {
                blend_enable: false,
                src_color_blend_factor: dacite::core::BlendFactor::One,
                dst_color_blend_factor: dacite::core::BlendFactor::Zero,
                color_blend_op: dacite::core::BlendOp::Add,
                src_alpha_blend_factor: dacite::core::BlendFactor::One,
                dst_alpha_blend_factor: dacite::core::BlendFactor::Zero,
                alpha_blend_op: dacite::core::BlendOp::Add,
                color_write_mask: dacite::core::COLOR_COMPONENT_R_BIT | dacite::core::COLOR_COMPONENT_G_BIT | dacite::core::COLOR_COMPONENT_B_BIT,
            }],
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            chain: None,
        }),
        dynamic_state: None,
        layout: layout.clone(),
        render_pass: render_pass.clone(),
        subpass: 0,
        base_pipeline: None,
        base_pipeline_index: None,
        chain: None,
    }];

    let pipelines = device.create_graphics_pipelines(None, &create_infos, None).map_err(|(e, _)| {
        println!("Failed to create pipeline ({})", e);
    })?;

    Ok(pipelines[0].clone())
}

pub fn create_command_pool(device: &dacite::core::Device,
                       queue_family_index: u32) -> Result<dacite::core::CommandPool, ()> {
    let create_info = dacite::core::CommandPoolCreateInfo {
        flags: dacite::core::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_family_index,
        chain: None,
    };

    device.create_command_pool(&create_info, None).map_err(|e| {
        println!("Failed to create command pool ({})", e);
    })
}

pub fn record_command_buffer(command_pool: &dacite::core::CommandPool,
                         pipeline: &dacite::core::Pipeline,
                         framebuffers: &[dacite::core::Framebuffer],
                         render_pass: &dacite::core::RenderPass,
                         extent: &dacite::core::Extent2D) -> Result<Vec<dacite::core::CommandBuffer>, ()> {
    let allocate_info = dacite::core::CommandBufferAllocateInfo {
        command_pool: command_pool.clone(),
        level: dacite::core::CommandBufferLevel::Primary,
        command_buffer_count: framebuffers.len() as u32,
        chain: None,
    };

    let command_buffers = dacite::core::CommandPool::allocate_command_buffers(&allocate_info).map_err(|e| {
        println!("Failed to allocate command buffers ({})", e);
    })?;

    for (command_buffer, framebuffer) in command_buffers.iter().zip(framebuffers.iter()) {
        let begin_info = dacite::core::CommandBufferBeginInfo {
            flags: dacite::core::CommandBufferUsageFlags::empty(),
            inheritance_info: None,
            chain: None,
        };

        command_buffer.begin(&begin_info).map_err(|e| {
            println!("Failed to begin command buffer ({})", e);
        })?;

        let begin_info = dacite::core::RenderPassBeginInfo {
            render_pass: render_pass.clone(),
            framebuffer: framebuffer.clone(),
            render_area: dacite::core::Rect2D::new(dacite::core::Offset2D::zero(), *extent),
            clear_values: vec![dacite::core::ClearValue::Color(dacite::core::ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0]))],
            chain: None,
        };

        command_buffer.begin_render_pass(&begin_info, dacite::core::SubpassContents::Inline);
        command_buffer.bind_pipeline(dacite::core::PipelineBindPoint::Graphics, pipeline);
        command_buffer.draw(3, 1, 0, 0);

        command_buffer.end_render_pass();
        command_buffer.end().map_err(|e| {
            println!("Failed to record command buffer ({})", e);
        })?;
    }

    Ok(command_buffers)
}

pub fn create_semaphores(device: &dacite::core::Device) -> Result<(dacite::core::Semaphore, dacite::core::Semaphore), ()> {
    let create_info = dacite::core::SemaphoreCreateInfo {
        flags: dacite::core::SemaphoreCreateFlags::empty(),
        chain: None,
    };

    let image_acquired = device.create_semaphore(&create_info, None).map_err(|e| {
        println!("Failed to create semaphore ({})", e);
    })?;

    let image_rendered = device.create_semaphore(&create_info, None).map_err(|e| {
        println!("Failed to create semaphore ({})", e);
    })?;

    Ok((image_acquired, image_rendered))
}
