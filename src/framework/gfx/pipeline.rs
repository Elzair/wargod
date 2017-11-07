use std::sync::Arc;

use vulkano::pipeline as vkp;

use super::core;

struct Pipeline {
    pub id: Arc<vkp::GraphicsPipelineAbstract + Send + Sync>,
}

struct PipelineBuilder

impl Pipeline {
    pub fn new(core: Arc<core::Core>,
    params: vkp::G) -> Result<Pipeline, ()> {
    } 
}

