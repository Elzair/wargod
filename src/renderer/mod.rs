// extern crate dacite;
// extern crate dacite_winit;
// extern crate winit;

use std::time::Duration;
use window;
use dacite::core as dc;
use dacite::khr_swapchain::{AcquireNextImageResultKhr, PresentInfoKhr};

pub mod core;

pub struct Renderer {
    pub image_rendered: dc::Semaphore,
    pub image_acquired: dc::Semaphore,
    pub command_buffers: Vec<dc::CommandBuffer>,
    pub command_pool: dc::CommandPool,
    pub pipeline: dc::Pipeline,
    pub core: core::Core,
}

impl Renderer {
    pub fn new(window: &window::Window) -> Result<Self, ()> {
        let core = core::Core::new(window)?;
        let pipeline = create_pipeline(&core.device.device,
                                       &core.swapchain.render_pass,
                                       &window.extent)?;
        let command_pool = create_command_pool(&core.device.device,
                                               core.device.queue_family_indices.graphics)?;
        let command_buffers = record_command_buffer(&command_pool,
                                                    &pipeline,
                                                    &core.swapchain.framebuffers,
                                                    &core.swapchain.render_pass,
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
        let next_image_res = self.core.swapchain.swapchain.acquire_next_image_khr(dc::Timeout::Some(Duration::from_millis(17)),
                                                                                  Some(&self.image_acquired), None).map_err(|e| {
            println!("Failed to acquire next image ({})", e);
        })?;

        let next_image = match next_image_res {
            AcquireNextImageResultKhr::Index(idx) |
            AcquireNextImageResultKhr::Suboptimal(idx) => idx,
            AcquireNextImageResultKhr::Timeout |
            AcquireNextImageResultKhr::NotReady => return Ok(()),
        };

        let submit_infos = vec![dc::SubmitInfo {
            wait_semaphores: vec![self.image_acquired.clone()],
            wait_dst_stage_mask: vec![dc::PIPELINE_STAGE_TOP_OF_PIPE_BIT],
            command_buffers: vec![self.command_buffers[next_image].clone()],
            signal_semaphores: vec![self.image_rendered.clone()],
            chain: None,
        }];

        self.core.device.graphics_queue.submit(Some(&submit_infos), None).map_err(|e| {
            println!("Failed to submit command buffer ({})", e);
        })?;

        let mut present_info = PresentInfoKhr {
            wait_semaphores: vec![self.image_rendered.clone()],
            swapchains: vec![self.core.swapchain.swapchain.clone()],
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
    device: &dc::Device
) -> Result<dc::ShaderModule, ()> {
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

    let create_info = dc::ShaderModuleCreateInfo {
        flags: dc::ShaderModuleCreateFlags::empty(),
        code: vertex_shader_bytes.to_vec(),
        chain: None,
    };

    device.create_shader_module(&create_info, None).map_err(|e| {
        println!("Failed to create vertex shader module ({})", e);
    })
}

pub fn create_fragment_shader(
    device: &dc::Device
) -> Result<dc::ShaderModule, ()> {
    let fragment_shader_bytes = glsl_fs!{r#"
        #version 450

        layout(location = 0) in vec3 fragColor;

        layout(location = 0) out vec4 outColor;

        void main() {
            outColor = vec4(fragColor, 1.0);
        }
    "#};

    let create_info = dc::ShaderModuleCreateInfo {
        flags: dc::ShaderModuleCreateFlags::empty(),
        code: fragment_shader_bytes.to_vec(),
        chain: None,
    };

    device.create_shader_module(&create_info, None).map_err(|e| {
        println!("Failed to create fragment shader module ({})", e);
    })
}

pub fn create_pipeline_layout(
    device: &dc::Device
) -> Result<dc::PipelineLayout, ()> {
    let create_info = dc::PipelineLayoutCreateInfo {
        flags: dc::PipelineLayoutCreateFlags::empty(),
        set_layouts: vec![],
        push_constant_ranges: vec![],
        chain: None,
    };

    device.create_pipeline_layout(&create_info, None).map_err(|e| {
        println!("Failed to create pipeline layout ({})", e);
    })
}

pub fn create_pipeline(
    device: &dc::Device,
    render_pass: &dc::RenderPass,
    extent: &dc::Extent2D
) -> Result<dc::Pipeline, ()> {
    let vertex_shader = create_vertex_shader(device)?;
    let fragment_shader = create_fragment_shader(device)?;
    let layout = create_pipeline_layout(device)?;

    let create_infos = vec![dc::GraphicsPipelineCreateInfo {
        flags: dc::PipelineCreateFlags::empty(),
        stages: vec![
            dc::PipelineShaderStageCreateInfo {
                flags: dc::PipelineShaderStageCreateFlags::empty(),
                stage: dc::SHADER_STAGE_VERTEX_BIT,
                module: vertex_shader.clone(),
                name: "main".to_owned(),
                specialization_info: None,
                chain: None,
            },
            dc::PipelineShaderStageCreateInfo {
                flags: dc::PipelineShaderStageCreateFlags::empty(),
                stage: dc::SHADER_STAGE_FRAGMENT_BIT,
                module: fragment_shader.clone(),
                name: "main".to_owned(),
                specialization_info: None,
                chain: None,
            },
        ],
        vertex_input_state: dc::PipelineVertexInputStateCreateInfo {
            flags: dc::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_descriptions: vec![],
            vertex_attribute_descriptions: vec![],
            chain: None,
        },
        input_assembly_state: dc::PipelineInputAssemblyStateCreateInfo {
            flags: dc::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: dc::PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            chain: None,
        },
        tessellation_state: None,
        viewport_state: Some(dc::PipelineViewportStateCreateInfo {
            flags: dc::PipelineViewportStateCreateFlags::empty(),
            viewports: vec![dc::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
            scissors: vec![dc::Rect2D::new(dc::Offset2D::zero(),
                                                     *extent)],
            chain: None,
        }),
        rasterization_state: dc::PipelineRasterizationStateCreateInfo {
            flags: dc::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: dc::PolygonMode::Fill,
            cull_mode: dc::CULL_MODE_NONE,
            front_face: dc::FrontFace::Clockwise,
            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
            chain: None,
        },
        multisample_state: Some(dc::PipelineMultisampleStateCreateInfo {
            flags: dc::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: dc::SAMPLE_COUNT_1_BIT,
            sample_shading_enable: false,
            min_sample_shading: 1.0,
            sample_mask: vec![],
            alpha_to_coverage_enable: false,
            alpha_to_one_enable: false,
            chain: None,
        }),
        depth_stencil_state: None,
        color_blend_state: Some(dc::PipelineColorBlendStateCreateInfo {
            flags: dc::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: false,
            logic_op: dc::LogicOp::Copy,
            attachments: vec![dc::PipelineColorBlendAttachmentState {
                blend_enable: false,
                src_color_blend_factor: dc::BlendFactor::One,
                dst_color_blend_factor: dc::BlendFactor::Zero,
                color_blend_op: dc::BlendOp::Add,
                src_alpha_blend_factor: dc::BlendFactor::One,
                dst_alpha_blend_factor: dc::BlendFactor::Zero,
                alpha_blend_op: dc::BlendOp::Add,
                color_write_mask: dc::COLOR_COMPONENT_R_BIT | dc::COLOR_COMPONENT_G_BIT | dc::COLOR_COMPONENT_B_BIT,
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
    device: &dc::Device,
    queue_family_index: u32
) -> Result<dc::CommandPool, ()> {
    let create_info = dc::CommandPoolCreateInfo {
        flags: dc::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_family_index,
        chain: None,
    };

    device.create_command_pool(&create_info, None).map_err(|e| {
        println!("Failed to create command pool ({})", e);
    })
}

pub fn record_command_buffer(
    command_pool: &dc::CommandPool,
    pipeline: &dc::Pipeline,
    framebuffers: &[dc::Framebuffer],
    render_pass: &dc::RenderPass,
    extent: &dc::Extent2D
) -> Result<Vec<dc::CommandBuffer>, ()> {
    let allocate_info = dc::CommandBufferAllocateInfo {
        command_pool: command_pool.clone(),
        level: dc::CommandBufferLevel::Primary,
        command_buffer_count: framebuffers.len() as u32,
        chain: None,
    };

    let command_buffers = dc::CommandPool::allocate_command_buffers(&allocate_info).map_err(|e| {
        println!("Failed to allocate command buffers ({})", e);
    })?;

    for (command_buffer, framebuffer) in command_buffers.iter().zip(framebuffers.iter()) {
        let begin_info = dc::CommandBufferBeginInfo {
            flags: dc::CommandBufferUsageFlags::empty(),
            inheritance_info: None,
            chain: None,
        };

        command_buffer.begin(&begin_info).map_err(|e| {
            println!("Failed to begin command buffer ({})", e);
        })?;

        let begin_info = dc::RenderPassBeginInfo {
            render_pass: render_pass.clone(),
            framebuffer: framebuffer.clone(),
            render_area: dc::Rect2D::new(dc::Offset2D::zero(), *extent),
            clear_values: vec![dc::ClearValue::Color(dc::ClearColorValue::Float32([0.0, 0.0, 0.0, 1.0]))],
            chain: None,
        };

        command_buffer.begin_render_pass(&begin_info, dc::SubpassContents::Inline);
        command_buffer.bind_pipeline(dc::PipelineBindPoint::Graphics, pipeline);
        command_buffer.draw(3, 1, 0, 0);

        command_buffer.end_render_pass();
        command_buffer.end().map_err(|e| {
            println!("Failed to record command buffer ({})", e);
        })?;
    }

    Ok(command_buffers)
}

pub fn create_semaphores(
    device: &dc::Device
) -> Result<(dc::Semaphore, dc::Semaphore), ()> {
    let create_info = dc::SemaphoreCreateInfo {
        flags: dc::SemaphoreCreateFlags::empty(),
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
