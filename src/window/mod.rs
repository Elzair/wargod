extern crate dacite;
extern crate winit;

pub struct Window {
    pub events_loop: winit::EventsLoop,
    pub window:      winit::Window,
    pub extent:      dacite::core::Extent2D,
}

impl Window {
    pub fn new(extent: dacite::core::Extent2D) -> Result<Self, ()> {
        let events_loop = winit::EventsLoop::new();

        let window = winit::WindowBuilder::new()
            .with_title("dacite triangle example")
            .with_dimensions(extent.width, extent.height)
            .with_min_dimensions(extent.width, extent.height)
            .with_max_dimensions(extent.width, extent.height)
            .with_visibility(false)
            .build(&events_loop);

        let window = window.map_err(|e| {
            match e {
                winit::CreationError::OsError(e) =>
                    println!("Failed to create window ({})", e),
                winit::CreationError::NotSupported =>
                    println!("Failed to create window (not supported)"),
            }
        })?;

        Ok(Window {
            events_loop: events_loop,
            window:      window,
            extent:      extent,
        })
    }
}
