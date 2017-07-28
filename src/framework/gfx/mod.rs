use winit;
use vulkano;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;

use std::sync::{Arc,RwLock};
use std::mem;

pub use vulkano::swapchain::{AcquireError, SwapchainAcquireFuture};

mod device;

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub struct Core {
    pub framebuffers: RwLock<Vec<Arc<vulkano::framebuffer::FramebufferAbstract + Send + Sync>>>,
    pub render_pass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    pub depth_buffer: RwLock<Arc<vulkano::image::attachment::AttachmentImage<vulkano::format::D16Unorm>>>,
    pub swapchain_images: RwLock<Vec<Arc<vulkano::image::swapchain::SwapchainImage>>>,
    pub swapchain: RwLock<Arc<vulkano::swapchain::Swapchain>>,
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

        // Create Swapchain
        
        let dims = [width, height];

        let (swapchain, swapchain_images) = {
            let usage = surface_capabilities.supported_usage_flags;
            let format = surface_capabilities.supported_formats[0].0;

            vulkano::swapchain::Swapchain::new(
                device.clone(),
                window.surface().clone(),
                surface_capabilities.min_image_count,
                format,
                dims,
                1,
                usage,
                &queue,
                vulkano::swapchain::SurfaceTransform::Identity,
                vulkano::swapchain::CompositeAlpha::Opaque,
                vulkano::swapchain::PresentMode::Fifo,
                true,
                None
            ).expect("failed to create swapchain")
        };

        // Create Depth Buffer

        let depth_buffer = vulkano::image::attachment::AttachmentImage::transient(
            device.clone(),
            dims,
            vulkano::format::D16Unorm
        ).unwrap();

        // Create Render Pass

        let render_pass = Arc::new(
            single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: vulkano::format::Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            ).unwrap()
        );

        // Create Framebuffers

        let framebuffers = swapchain_images.iter().map(|image| {
            let fb = vulkano::framebuffer::Framebuffer::start(render_pass.clone())
                     .add(image.clone()).unwrap()
                     .add(depth_buffer.clone()).unwrap()
                     .build().unwrap();
            Arc::new(fb) as Arc<vulkano::framebuffer::FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        // Return Core part of GFX
        
        Ok(Core {
            framebuffers: RwLock::new(framebuffers),
            depth_buffer: RwLock::new(depth_buffer),
            render_pass: render_pass,
            swapchain_images: RwLock::new(swapchain_images),
            swapchain: RwLock::new(swapchain),
            surface_capabilities: surface_capabilities,
            queue: queue,
            device: device,
            dimensions: RwLock::new(Dimensions {width: width,
                                                height: height}),
            window: window,
        })
    }

    pub fn acquire_next_framebuffer(&mut self) -> Result<(usize, SwapchainAcquireFuture), AcquireError> {
        vulkano::swapchain::acquire_next_image(self.swapchain.read().unwrap().clone(), None)
        // let mut swapchain_recreated = false;


        
        // loop {
        //     let (image_num, acquire_future): (usize, vulkano::swapchain::SwapchainAcquireFuture) = match vulkano::swapchain::acquire_next_image(
        //         self.swapchain.read().unwrap().clone(),
        //         None
        //     ) {
        //         Ok((idx, future)) => return Ok((idx, swapchain_recreated, future)),
        //         Err(vulkano::swapchain::AcquireError::OutOfDate) => {
        //             // Recreate swapchain
        //             self.recreate_swapchain();
        //             swapchain_recreated = true;
        //             continue;
        //         },
        //         Err(err) => return Err(err)
        //     };
        // }
    }

    pub fn recreate_swapchain(&mut self) {
        println!("Recreating swapchain!");
        loop {
            println!("Trying again!");
            let (new_width, new_height) = self.window.window()
                .get_inner_size_pixels().unwrap();

            let dims = [new_width, new_height];

            let (new_swapchain, new_images) = match self.swapchain.read().unwrap()
                .recreate_with_dimension(dims) {
                    Ok(r) => r,
                    Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                        continue;
                    },
                    Err(err) => panic!("{:?}", err)
                };

            let new_depth_buffer = vulkano::image::attachment::AttachmentImage
                ::transient(
                    self.device.clone(),
                    dims,
                    vulkano::format::D16Unorm
                ).unwrap();

            let new_framebuffers = new_images.iter().map(|image| {
                let fb = vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
                    .add(image.clone()).unwrap()
                    .add(new_depth_buffer.clone()).unwrap()
                    .build().unwrap();
                Arc::new(fb) as Arc<vulkano::framebuffer::FramebufferAbstract + Send + Sync>
            }).collect::<Vec<_>>();

            let mut dimensions_ref = self.dimensions.write().unwrap();
            let mut swapchain_ref = self.swapchain.write().unwrap();
            let mut swapchain_images_ref = self.swapchain_images.write().unwrap();
            let mut depth_buffer_ref = self.depth_buffer.write().unwrap();
            let mut framebuffers_ref = self.framebuffers.write().unwrap();
            *framebuffers_ref = new_framebuffers;
            *depth_buffer_ref = new_depth_buffer;
            *swapchain_images_ref = new_images;
            *swapchain_ref = new_swapchain;
            *dimensions_ref = {
                Dimensions { width: new_width, height: new_height }
            };

            break;
        }
    }
}
