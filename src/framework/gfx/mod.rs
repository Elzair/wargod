use winit;
use vulkano;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;

use std::sync::{Arc,RwLock};

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub struct Core {
    pub surface_capabilities: vulkano::swapchain::Capabilities,
    pub queue: Arc<vulkano::device::Queue>,
    pub device: Arc<vulkano::device::Device>,
    pub dimensions: RwLock<Dimensions>,
    pub window: vulkano_win::Window,
}

impl Core {
    pub fn new(events_loop: &winit::EventsLoop) -> Result<Core, ()> {
        let extensions = vulkano_win::required_extensions();
        let instance = vulkano::instance::Instance::new(None, &extensions, None)
            .expect("failed to create instance");

        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
            .next().expect("no device available");
        println!("Using device: {} (type: {:?})", physical.name(), physical.ty());


        let window = winit::WindowBuilder::new().build_vk_surface(events_loop,
                                                                  instance.clone()).unwrap();

        let (width, height) = window.window().get_inner_size_pixels().unwrap();

        let queue = physical.queue_families().find(|&q| q.supports_graphics() &&
                                                   window.surface().is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let (device, mut queues) = vulkano::device::Device::new(physical,
                                                                physical.supported_features(),
                                                                &device_ext,
                                                                [(queue, 0.5)].iter().cloned())
            .expect("failed to create device");
        
        let queue = queues.next().unwrap();

        let surface_capabilities = window.surface().capabilities(physical).expect("failed to get surface capabilities");

        Ok(Core {
            surface_capabilities: surface_capabilities,
            queue: queue,
            device: device,
            dimensions: RwLock::new(Dimensions { width: width, height: height}),
            window: window,
        })
    }
}
