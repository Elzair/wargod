use vulkano_win;
use vulkano::device as vkd;
use vulkano::swapchain as vks;
use vulkano::framebuffer as vkfb;
use vulkano::image as vkim;
use vulkano::format as vkfmt;

use std::sync::{Arc,RwLock};
use vulkano::swapchain::{AcquireError, SwapchainAcquireFuture};

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub struct Swapchain {
    pub framebuffers: Vec<Arc<vkfb::FramebufferAbstract + Send + Sync>>,
    pub depth_buffer: Arc<vkim::attachment::AttachmentImage<vkfmt::D16Unorm>>,
    pub images: Vec<Arc<vkim::swapchain::SwapchainImage>>,
    pub id: Arc<vks::Swapchain>,
}

impl Swapchain {
    pub fn new(device: Arc<vkd::Device>,
               queue: Arc<vkd::Queue>,
               window: Arc<vulkano_win::Window>,
               render_pass: Arc<vkfb::RenderPassAbstract + Send + Sync>,
               surface_capabilities: Arc<vks::Capabilities>,
               width: u32,
               height: u32) -> Result<Arc<RwLock<Swapchain>>, ()> {
        // Create Swapchain
        
        let dims = [width, height];

        let (swapchain, images) = {
            let usage = surface_capabilities.supported_usage_flags;
            let format = surface_capabilities.supported_formats[0].0;

            vks::Swapchain::new(
                device.clone(),
                window.clone().surface().clone(),
                surface_capabilities.min_image_count,
                format,
                dims,
                1,
                usage,
                &queue,
                vks::SurfaceTransform::Identity,
                vks::CompositeAlpha::Opaque,
                vks::PresentMode::Fifo,
                true,
                None
            ).expect("failed to create swapchain")
        };

        // Create Depth Buffer

        let depth_buffer = vkim::attachment::AttachmentImage::transient(
            device.clone(),
            dims,
            vkfmt::D16Unorm
        ).unwrap();

        // Create Framebuffers

        let framebuffers = images.iter().map(|image| {
            let fb = vkfb::Framebuffer::start(render_pass.clone())
                     .add(image.clone()).unwrap()
                     .add(depth_buffer.clone()).unwrap()
                     .build().unwrap();
            Arc::new(fb) as Arc<vkfb::FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        Ok(Arc::new(RwLock::new(Swapchain {
            framebuffers: framebuffers,
            depth_buffer: depth_buffer,
            images: images,
            id: swapchain,
        })))
    }

    pub fn acquire_next_framebuffer(&self) -> Result<(usize, Arc<vkfb::FramebufferAbstract + Send + Sync>, SwapchainAcquireFuture), AcquireError> {
        match vks::acquire_next_image(self.id.clone(), None) {
            Ok((idx, future)) => Ok((idx, self.framebuffers[idx].clone(), future)),
            Err(err) => Err(err),
        }
    }

    pub fn refresh(&mut self,
                   device: Arc<vkd::Device>,
                   render_pass: Arc<vkfb::RenderPassAbstract + Send + Sync>,
                   width: u32,
                   height: u32) -> bool {
        let dims = [width, height];

        let (new_swapchain, new_images) = match self.id.recreate_with_dimension(dims) {
            Ok(r) => r,
            // This seems to happen when the user is manually resizing the window.
            Err(vks::SwapchainCreationError::UnsupportedDimensions) => {
                return false;
            },
            Err(err) => panic!("{:?}", err)
        };

        let new_depth_buffer = vkim::attachment::AttachmentImage
            ::transient(
                device.clone(),
                dims,
                vkfmt::D16Unorm
            ).unwrap();

        let new_framebuffers = new_images.iter().map(|image| {
            let fb = vkfb::Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .add(new_depth_buffer.clone()).unwrap()
                .build().unwrap();
            Arc::new(fb) as Arc<vkfb::FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        self.framebuffers = new_framebuffers;
        self.depth_buffer = new_depth_buffer;
        self.images = new_images;
        self.id = new_swapchain;

        true
    }
}
