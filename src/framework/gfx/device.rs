use vulkano;
use std::sync::Arc;

pub fn get_required_features() -> vulkano::instance::Features {
    vulkano::instance::Features {
        tessellation_shader: true,
        .. vulkano::instance::Features::none()
    }
}

pub fn find_suitable_devices(instance: &Arc<vulkano::instance::Instance>,
                             required_features: &vulkano::instance::Features ) 
                            -> Vec<(String, usize)> {
    vulkano::instance::PhysicalDevice::enumerate(&instance)
        .filter(|ph| ph.supported_features().superset_of(required_features))
        .map(|ph| (ph.name(), ph.index()))
        .collect::<Vec<(String, usize)>>()
}

pub fn init_physical_device(instance: &Arc<vulkano::instance::Instance>,
                        index: Option<usize>)
                        -> Option<vulkano::instance::PhysicalDevice> {
    match index {
        Some(idx) => vulkano::instance::PhysicalDevice::from_index(instance, idx),
        None => None
    }
}
