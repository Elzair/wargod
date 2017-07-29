use winit;
use vulkano;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;

use std::sync::{Arc,RwLock};

use vulkano::instance as vki;
use vulkano::device as vkd;
use vulkano::swapchain as vks;
use vulkano::framebuffer as vkfb;
use vulkano::image as vkim;
use vulkano::format as vkfmt;

pub use vulkano::swapchain::{AcquireError, SwapchainAcquireFuture};

pub mod swapchain;

pub use self::swapchain::Dimensions;

pub struct Core {
    pub swapchain: Arc<RwLock<swapchain::Swapchain>>,
    //pub framebuffers: RwLock<Vec<Arc<vkfb::FramebufferAbstract + Send + Sync>>>,
    pub render_pass: Arc<vkfb::RenderPassAbstract + Send + Sync>,
    //pub depth_buffer: RwLock<Arc<vkim::attachment::AttachmentImage<vkfmt::D16Unorm>>>,
    //pub swapchain_images: RwLock<Vec<Arc<vkim::swapchain::SwapchainImage>>>,
    //pub swapchain: RwLock<Arc<vks::Swapchain>>,
    pub surface_capabilities: Arc<vks::Capabilities>,
    pub queue: Arc<vkd::Queue>,
    pub device: Arc<vkd::Device>,
    pub dimensions: RwLock<Dimensions>,
    pub window: Arc<vulkano_win::Window>,
}

impl Core {
    pub fn new(events_loop: &winit::EventsLoop) -> Result<Arc<Core>, ()> {
        // Create Instance
        
        let extensions = vulkano_win::required_extensions();
        let instance = vki::Instance::new(None, &extensions, None)
            .expect("failed to create instance");

        // Create window
        
        let window = Arc::new(winit::WindowBuilder::new().build_vk_surface(
            events_loop,
            instance.clone()
        ).unwrap());

        let (width, height) = window.window().get_inner_size_pixels().unwrap();

        // Find Physical Device
        
        let required_features = get_required_features();
        let (_, idx) = find_suitable_devices(&instance, &required_features)
            .into_iter().next()
            .expect("No suitable devices available");

        let physical = init_physical_device(&instance, Some(idx)).unwrap();

        println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

        let surface_capabilities = Arc::new(window.surface().capabilities(physical)
            .expect("failed to get surface capabilities"));

        let queue = physical.queue_families().find(|&q| {
            q.supports_graphics() &&
                window.surface().is_supported(q).unwrap_or(false)
        }).expect("Could not find a graphical queue family");

        // Create Logical Device

        let device_extensions = vkd::DeviceExtensions {
            khr_swapchain: true,
            .. vkd::DeviceExtensions::none()
        };

        let (device, mut queues) = vkd::Device::new(
            physical,
            &required_features,
            &device_extensions,
            [(queue, 0.5)].iter().cloned()
        ).expect("failed to create device");
        
        // Create Queues

        let queue = queues.next().unwrap();

        // // Create Swapchain
        
        // let dims = [width, height];

        // let (swapchain, swapchain_images) = {
        //     let usage = surface_capabilities.supported_usage_flags;
        //     let format = surface_capabilities.supported_formats[0].0;

        //     vks::Swapchain::new(
        //         device.clone(),
        //         window.surface().clone(),
        //         surface_capabilities.min_image_count,
        //         format,
        //         dims,
        //         1,
        //         usage,
        //         &queue,
        //         vks::SurfaceTransform::Identity,
        //         vks::CompositeAlpha::Opaque,
        //         vks::PresentMode::Fifo,
        //         true,
        //         None
        //     ).expect("failed to create swapchain")
        // };

        // // Create Depth Buffer

        // let depth_buffer = vkim::attachment::AttachmentImage::transient(
        //     device.clone(),
        //     dims,
        //     vkfmt::D16Unorm
        // ).unwrap();

        // Create Render Pass

        let render_pass = Arc::new(
            single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: surface_capabilities.supported_formats[0].0,
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: vkfmt::Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            ).unwrap()
        );

        let swapchain = swapchain::Swapchain::new(device.clone(),
                                                  queue.clone(),
                                                  window.clone(),
                                                  render_pass.clone(),
                                                  surface_capabilities.clone(),
                                                  width,
                                                  height).unwrap();

        // Create Framebuffers

        // let framebuffers = swapchain_images.iter().map(|image| {
        //     let fb = vkfb::Framebuffer::start(render_pass.clone())
        //              .add(image.clone()).unwrap()
        //              .add(depth_buffer.clone()).unwrap()
        //              .build().unwrap();
        //     Arc::new(fb) as Arc<vkfb::FramebufferAbstract + Send + Sync>
        // }).collect::<Vec<_>>();

        // Return Core part of GFX
        
        Ok(Arc::new(Core {
            swapchain: swapchain,
            //framebuffers: RwLock::new(framebuffers),
            //depth_buffer: RwLock::new(depth_buffer),
            render_pass: render_pass,
            //swapchain_images: RwLock::new(swapchain_images),
            //swapchain: RwLock::new(swapchain),
            surface_capabilities: surface_capabilities,
            queue: queue,
            device: device,
            dimensions: RwLock::new(Dimensions {width: width,
                                                height: height}),
            window: window,
        }))
    }

    pub fn acquire_next_framebuffer(&self) -> Result<(usize, Arc<vkfb::FramebufferAbstract + Send + Sync>, SwapchainAcquireFuture), AcquireError> {
        //vks::acquire_next_image(self.swapchain.read().unwrap().clone(), None)
        self.swapchain.read().unwrap().acquire_next_framebuffer()
    }

    pub fn recreate_swapchain(&self) {
        let mut done = false;
        
        while !done {
            let (new_width, new_height) = self.window.window()
                .get_inner_size_pixels().unwrap();

            done = self.swapchain.write().unwrap()
                .refresh(self.device.clone(),
                         self.render_pass.clone(),
                         new_width,
                         new_height);

            let mut dimensions_ref = self.dimensions.write().unwrap();
            dimensions_ref.width = new_width;
            dimensions_ref.height = new_height;
        }
    }
}

fn get_required_features() -> vki::Features {
    vki::Features {
        tessellation_shader: true,
        .. vki::Features::none()
    }
}

fn find_suitable_devices(instance: &Arc<vki::Instance>,
                         required_features: &vki::Features) 
                            -> Vec<(String, usize)> {
    vki::PhysicalDevice::enumerate(&instance)
        .filter(|ph| ph.supported_features().superset_of(required_features))
        .map(|ph| (ph.name(), ph.index()))
        .collect::<Vec<(String, usize)>>()
}

fn init_physical_device(instance: &Arc<vki::Instance>,
                        index: Option<usize>)
                        -> Option<vki::PhysicalDevice> {
    match index {
        Some(idx) => vki::PhysicalDevice::from_index(instance, idx),
        None => None
    }
}
