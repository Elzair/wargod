use window;

pub mod device;
pub mod swapchain;

pub struct Core {
    pub swapchain: swapchain::Internal,
    pub device: device::Internal,
}

impl Core {
    pub fn new(window: &window::Window) -> Result<Self, ()> {
        let device = device::Internal::new(window)?;

        let swapchain = swapchain::Internal::new(&device, &window.extent)?;
     
        Ok(Core {
            swapchain: swapchain,
            device: device,
        })
    }
}


