extern crate dacite;
extern crate dacite_winit;
extern crate winit;

pub mod init;

use dacite_winit::WindowExt;
use std::cmp;
use std::time::Duration;

fn render(graphics_queue: &dacite::core::Queue,
          present_queue: &dacite::core::Queue,
          command_buffers: &[dacite::core::CommandBuffer],
          swapchain: &dacite::khr_swapchain::SwapchainKhr,
          image_acquired: &dacite::core::Semaphore,
          image_rendered: &dacite::core::Semaphore) -> Result<(), ()> {
    let next_image_res = swapchain.acquire_next_image_khr(dacite::core::Timeout::Some(Duration::from_millis(17)), Some(image_acquired), None).map_err(|e| {
        println!("Failed to acquire next image ({})", e);
    })?;

    let next_image = match next_image_res {
        dacite::khr_swapchain::AcquireNextImageResultKhr::Index(idx) |
        dacite::khr_swapchain::AcquireNextImageResultKhr::Suboptimal(idx) => idx,
        dacite::khr_swapchain::AcquireNextImageResultKhr::Timeout |
        dacite::khr_swapchain::AcquireNextImageResultKhr::NotReady => return Ok(()),
    };

    let submit_infos = vec![dacite::core::SubmitInfo {
        wait_semaphores: vec![image_acquired.clone()],
        wait_dst_stage_mask: vec![dacite::core::PIPELINE_STAGE_TOP_OF_PIPE_BIT],
        command_buffers: vec![command_buffers[next_image].clone()],
        signal_semaphores: vec![image_rendered.clone()],
        chain: None,
    }];

    graphics_queue.submit(Some(&submit_infos), None).map_err(|e| {
        println!("Failed to submit command buffer ({})", e);
    })?;

    let mut present_info = dacite::khr_swapchain::PresentInfoKhr {
        wait_semaphores: vec![image_rendered.clone()],
        swapchains: vec![swapchain.clone()],
        image_indices: vec![next_image as u32],
        results: None,
        chain: None,
    };

    present_queue.queue_present_khr(&mut present_info).map_err(|e| {
        println!("Failed to present image ({})", e);
    })?;

    Ok(())
}

pub fn real_main() -> Result<(), ()> {
    let preferred_extent = dacite::core::Extent2D::new(800, 600);

    let init::Window {
        mut events_loop,
        window,
    } = init::create_window(&preferred_extent)?;

    let instance_extensions = init::compute_instance_extensions(&window)?;
    let instance = init::create_instance(instance_extensions)?;

    let surface = window.create_surface(&instance, dacite_winit::SurfaceCreateFlags::empty(), None).map_err(|e| match e {
        dacite_winit::Error::Unsupported => println!("The windowing system is not supported"),
        dacite_winit::Error::VulkanError(e) => println!("Failed to create surface ({})", e),
    })?;

    let init::DeviceSettings {
        physical_device,
        queue_family_indices,
        device_extensions,
    } = init::find_suitable_device(&instance, &surface)?;

    let device = init::create_device(&physical_device, device_extensions, &queue_family_indices)?;
    let graphics_queue = device.get_queue(queue_family_indices.graphics, 0);
    let present_queue = device.get_queue(queue_family_indices.present, 0);

    let init::SwapchainSettings {
        swapchain,
        extent,
        image_views: swapchain_image_views,
        format,
    } = init::create_swapchain(&physical_device, &device, &surface, &preferred_extent, &queue_family_indices)?;

    let render_pass = init::create_render_pass(&device, format)?;
    let framebuffers = init::create_framebuffers(&device, &swapchain_image_views, &render_pass, &extent)?;
    let pipeline = init::create_pipeline(&device, &render_pass, &extent)?;
    let command_pool = init::create_command_pool(&device, queue_family_indices.graphics)?;
    let command_buffers = init::record_command_buffer(&command_pool, &pipeline, &framebuffers, &render_pass, &extent)?;
    let (image_acquired, image_rendered) = init::create_semaphores(&device)?;

    window.show();

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } = event {
                running = false;
            }
        });

        render(&graphics_queue, &present_queue, &command_buffers, &swapchain, &image_acquired, &image_rendered)?;

        device.wait_idle().map_err(|e| {
            println!("Failed to wait for device becoming idle ({})", e);
        })?;
    }

    device.wait_idle().map_err(|e| {
        println!("Failed to wait for device becoming idle ({})", e);
    })?;

    Ok(())
}
