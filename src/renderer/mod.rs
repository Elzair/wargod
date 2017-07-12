extern crate dacite;
extern crate dacite_winit;
extern crate winit;

pub mod core;

use std::time::Duration;
use window;

pub struct Renderer {
    pub core: core::Core,
    pub pipeline: dacite::core::Pipeline,
    pub command_pool: dacite::core::CommandPool,
    pub command_buffers: Vec<dacite::core::CommandBuffer>,
    pub image_acquired: dacite::core::Semaphore,
    pub image_rendered: dacite::core::Semaphore,
}

impl Renderer {
    pub fn new(window: &window::Window) -> Result<Self, ()> {
        let core = core::Core::new(window)?;
        let pipeline = create_pipeline(&core.device.device,
                                       &core.render_pass,
                                       &window.extent)?;
        let command_pool = create_command_pool(&core.device.device,
                                               core.device.queue_family_indices.graphics)?;
        let command_buffers = record_command_buffer(&command_pool,
                                                    &pipeline,
                                                    &core.framebuffers,
                                                    &core.render_pass,
                                                    &window.extent)?;
        let (image_acquired, image_rendered) = create_semaphores(&core.device.device)?;

        window.window.show();

        Ok(Renderer {
            core: core,
            pipeline: pipeline,
            command_pool: command_pool,
            command_buffers: command_buffers,
            image_acquired: image_acquired,
            image_rendered: image_rendered,
        })
    }

    pub fn render(&self) -> Result<(), ()> {
        let next_image_res = self.core.swapchain.acquire_next_image_khr(dacite::core::Timeout::Some(Duration::from_millis(17)), Some(&self.image_acquired), None).map_err(|e| {
            println!("Failed to acquire next image ({})", e);
        })?;

        let next_image = match next_image_res {
            dacite::khr_swapchain::AcquireNextImageResultKhr::Index(idx) |
            dacite::khr_swapchain::AcquireNextImageResultKhr::Suboptimal(idx) => idx,
            dacite::khr_swapchain::AcquireNextImageResultKhr::Timeout |
            dacite::khr_swapchain::AcquireNextImageResultKhr::NotReady => return Ok(()),
        };

        let submit_infos = vec![dacite::core::SubmitInfo {
            wait_semaphores: vec![self.image_acquired.clone()],
            wait_dst_stage_mask: vec![dacite::core::PIPELINE_STAGE_TOP_OF_PIPE_BIT],
            command_buffers: vec![self.command_buffers[next_image].clone()],
            signal_semaphores: vec![self.image_rendered.clone()],
            chain: None,
        }];

        self.core.device.graphics_queue.submit(Some(&submit_infos), None).map_err(|e| {
            println!("Failed to submit command buffer ({})", e);
        })?;

        let mut present_info = dacite::khr_swapchain::PresentInfoKhr {
            wait_semaphores: vec![self.image_rendered.clone()],
            swapchains: vec![self.core.swapchain.clone()],
            image_indices: vec![next_image as u32],
            results: None,
            chain: None,
        };

        self.core.device.present_queue.queue_present_khr(&mut present_info).map_err(|e| {
            println!("Failed to present image ({})", e);
        })?;

        Ok(())
    }
}

pub fn create_vertex_shader(
    device: &dacite::core::Device
) -> Result<dacite::core::ShaderModule, ()> {
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

pub fn create_fragment_shader(
    device: &dacite::core::Device
) -> Result<dacite::core::ShaderModule, ()> {
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

pub fn create_pipeline_layout(
    device: &dacite::core::Device
) -> Result<dacite::core::PipelineLayout, ()> {
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

pub fn create_pipeline(
    device: &dacite::core::Device,
    render_pass: &dacite::core::RenderPass,
    extent: &dacite::core::Extent2D
) -> Result<dacite::core::Pipeline, ()> {
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
            scissors: vec![dacite::core::Rect2D::new(dacite::core::Offset2D::zero(),
                                                     *extent)],
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

    let pipelines = device.create_graphics_pipelines(None,
                                                     &create_infos,
                                                     None).map_err(|(e, _)| {
        println!("Failed to create pipeline ({})", e);
    })?;

    Ok(pipelines[0].clone())
}

pub fn create_command_pool(
    device: &dacite::core::Device,
    queue_family_index: u32
) -> Result<dacite::core::CommandPool, ()> {
    let create_info = dacite::core::CommandPoolCreateInfo {
        flags: dacite::core::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_family_index,
        chain: None,
    };

    device.create_command_pool(&create_info, None).map_err(|e| {
        println!("Failed to create command pool ({})", e);
    })
}

pub fn record_command_buffer(
    command_pool: &dacite::core::CommandPool,
    pipeline: &dacite::core::Pipeline,
    framebuffers: &[dacite::core::Framebuffer],
    render_pass: &dacite::core::RenderPass,
    extent: &dacite::core::Extent2D
) -> Result<Vec<dacite::core::CommandBuffer>, ()> {
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

pub fn create_semaphores(
    device: &dacite::core::Device
) -> Result<(dacite::core::Semaphore, dacite::core::Semaphore), ()> {
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
