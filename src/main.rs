extern crate dacite;
extern crate dacite_winit;
extern crate winit;
#[macro_use] extern crate glsl_to_spirv_macros;
#[macro_use] extern crate glsl_to_spirv_macros_impl;

pub mod renderer;
pub mod window;

fn main() {
    let preferred_extent = dacite::core::Extent2D::new(800, 600);
    let mut window = window::Window::new(preferred_extent).unwrap();
    let rend = renderer::init(&window).unwrap();
    
    let mut running = true;
    while running {
        window.events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } = event {
                running = false;
            }
        });

        renderer::render(&rend.graphics_queue,
                         &rend.present_queue,
                         &rend.command_buffers,
                         &rend.swapchain,
                         &rend.image_acquired,
                         &rend.image_rendered).unwrap();

        rend.device.wait_idle().map_err(|e| {
            println!("Failed to wait for device becoming idle ({})", e);
        }).unwrap();
    }

    rend.device.wait_idle().map_err(|e| {
        println!("Failed to wait for device becoming idle ({})", e);
    }).unwrap();

    // match renderer::real_main() {
    //     Ok(_) => process::exit(0),
    //     Err(_) => process::exit(1),
    // }
}
