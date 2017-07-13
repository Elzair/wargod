use window;

pub mod device;
pub mod swapchain;


pub struct Core {
    pub swapchain: swapchain::Internal,
    pub internal: device::Internal,
}

impl Core {
    pub fn new(window: &window::Window) -> Result<Self, ()> {
        let internal = device::Internal::new(window)?;

        let swapchain = swapchain::Internal::new(&internal, &window.extent)?;
     
        Ok(Core {
            swapchain: swapchain,
            internal: internal,
        })
    }
}


