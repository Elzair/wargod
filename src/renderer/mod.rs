use std::sync::Arc;

use super::framework::gfx;

//pub mod core;

pub struct Renderer {
    pub gfx: Arc<gfx::Core>,
}

impl Renderer {
    pub fn new(gfx: Arc<gfx::Core>) -> Result<Self, ()> {
        Ok(Renderer {
            gfx: gfx,
        })
    }

    pub fn render(&self) -> Result<(), ()> {
        Ok(())
    }
}
