use winit;
use vulkano;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;

use std::sync::{Arc,RwLock};

pub mod device;

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
        // Create Instance
        
        let extensions = vulkano_win::required_extensions();
        let instance = vulkano::instance::Instance::new(None, &extensions, None)
            .expect("failed to create instance");

        // Create window
        
        let window = winit::WindowBuilder::new().build_vk_surface(
            events_loop,
            instance.clone()
        ).unwrap();

        let (width, height) = window.window().get_inner_size_pixels().unwrap();

        // Find Physical Device
        
        let required_features = device::get_required_features();
        let (_, idx) = device::find_suitable_devices(&instance, &required_features)
            .into_iter().next()
            .expect("No suitable devices available");

        let physical = device::init_physical_device(&instance, Some(idx)).unwrap();

        println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

        let surface_capabilities = window.surface().capabilities(physical)
            .expect("failed to get surface capabilities");

        let queue = physical.queue_families().find(|&q| {
            q.supports_graphics() &&
                window.surface().is_supported(q).unwrap_or(false)
        }).expect("Could not find a graphical queue family");

        // Create Logical Device

        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let (device, mut queues) = vulkano::device::Device::new(
            physical,
            &required_features,
            &device_extensions,
            [(queue, 0.5)].iter().cloned()
        ).expect("failed to create device");
        
        // Create Queues

        let queue = queues.next().unwrap();

        // Return Core part of GFX
        
        Ok(Core {
            surface_capabilities: surface_capabilities,
            queue: queue,
            device: device,
            dimensions: RwLock::new(Dimensions {width: width,
                                                height: height}),
            window: window,
        })
    }
}
