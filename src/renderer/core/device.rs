extern crate winit;
extern crate dacite;
extern crate dacite_winit;
use dacite_winit::WindowExt;
use window;
use dacite::core as dc;

pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

pub struct DeviceSettings {
    pub physical_device: dc::PhysicalDevice,
    pub queue_family_indices: QueueFamilyIndices,
    pub device_extensions: dc::DeviceExtensions,
}

pub struct Internal {
    pub graphics_queue: dc::Queue,
    pub present_queue: dc::Queue,
    pub queue_family_indices: QueueFamilyIndices,
    pub device: dc::Device,
    pub surface: dacite::khr_surface::SurfaceKhr,
    pub physical_device: dc::PhysicalDevice,
    pub instance: dc::Instance,
}

impl Internal {
    pub fn new(window: &window::Window) -> Result<Self, ()> {
        let instance_extensions = compute_instance_extensions(&window.window)?;
        let instance = create_instance(instance_extensions)?;

        let surface = window.window.create_surface(&instance, dacite_winit::SurfaceCreateFlags::empty(), None).map_err(|e| match e {
            dacite_winit::Error::Unsupported => println!("The windowing system is not supported"),
            dacite_winit::Error::VulkanError(e) => println!("Failed to create surface ({})", e),
        })?;

        let DeviceSettings {
            physical_device,
            queue_family_indices,
            device_extensions,
        } = find_suitable_device(&instance, &surface)?;

        let device = create_device(&physical_device, device_extensions, &queue_family_indices)?;

        let graphics_queue = device.get_queue(queue_family_indices.graphics, 0);
        let present_queue = device.get_queue(queue_family_indices.present, 0);

        Ok(Internal {
            instance: instance,
            surface: surface,
            physical_device: physical_device,
            device: device,
            queue_family_indices: queue_family_indices,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
        })
    }
 }

fn compute_instance_extensions(
    window: &winit::Window
) -> Result<dc::InstanceExtensions, ()> {
    let available_extensions = dc::Instance::get_instance_extension_properties(None).map_err(|e| {
        println!("Failed to get instance extension properties ({})", e);
    })?;

    let mut required_extensions = window.get_required_extensions().map_err(|e| match e {
        dacite_winit::Error::Unsupported => println!("The windowing system is not supported"),
        dacite_winit::Error::VulkanError(e) => println!("Failed to get required extensions for the window ({})", e),
    })?;
    required_extensions.add_ext_debug_report(1);

    let missing_extensions = required_extensions.difference(&available_extensions);
    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    }
    else {
        for (name, spec_version) in missing_extensions.properties() {
            println!("Extension {} (revision {}) missing", name, spec_version);
        }

        Err(())
    }
}

fn create_instance(
    instance_extensions: dc::InstanceExtensions
) -> Result<dc::Instance, ()> {
    let application_info = dc::ApplicationInfo {
        application_name: Some("dacite triangle example".to_owned()),
        application_version: 0,
        engine_name: None,
        engine_version: 0,
        api_version: Some(dacite::DACITE_API_VERSION_1_0),
        chain: None,
    };

    let validation_layer = String::from("VK_LAYER_LUNARG_standard_validation");

    let create_info = dc::InstanceCreateInfo {
        flags: dc::InstanceCreateFlags::empty(),
        application_info: Some(application_info),
        enabled_layers: vec![validation_layer],
        enabled_extensions: instance_extensions,
        chain: None,
    };

    dc::Instance::create(&create_info, None).map_err(|e| {
        println!("Failed to create instance ({})", e);
    })
}

fn find_queue_family_indices(
    physical_device: &dc::PhysicalDevice,
    surface: &dacite::khr_surface::SurfaceKhr
) -> Result<QueueFamilyIndices, ()> {
    let mut graphics = None;
    let mut present = None;

    let queue_family_properties: Vec<_> = physical_device.get_queue_family_properties();
    for (index, queue_family_properties) in queue_family_properties.into_iter().enumerate() {
        if queue_family_properties.queue_count == 0 {
            continue;
        }

        if graphics.is_none() && queue_family_properties.queue_flags.contains(dc::QUEUE_GRAPHICS_BIT) {
            graphics = Some(index);
        }

        if present.is_none() {
            if let Ok(true) = physical_device.get_surface_support_khr(index as u32, surface) {
                present = Some(index);
            }
        }
    }

    if let (Some(graphics), Some(present)) = (graphics, present) {
        Ok(QueueFamilyIndices {
            graphics: graphics as u32,
            present: present as u32,
        })
    }
    else {
        Err(())
    }
}

fn check_device_extensions(
    physical_device: &dc::PhysicalDevice
) -> Result<dc::DeviceExtensions, ()> {
    let available_extensions = physical_device.get_device_extension_properties(None).map_err(|e| {
        println!("Failed to get device extension properties ({})", e);
    })?;

    let mut required_extensions = dc::DeviceExtensionsProperties::new();
    required_extensions.add_khr_swapchain(67);

    let missing_extensions = required_extensions.difference(&available_extensions);
    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    }
    else {
        for (name, spec_version) in missing_extensions.properties() {
            println!("Extension {} (revision {}) missing", name, spec_version);
        }

        Err(())
    }
}

fn check_device_suitability(
    physical_device: dc::PhysicalDevice,
    surface: &dacite::khr_surface::SurfaceKhr
) -> Result<DeviceSettings, ()> {
    let queue_family_indices = find_queue_family_indices(&physical_device, surface)?;
    let device_extensions = check_device_extensions(&physical_device)?;

    Ok(DeviceSettings {
        physical_device: physical_device,
        queue_family_indices: queue_family_indices,
        device_extensions: device_extensions,
    })
}

fn find_suitable_device(
    instance: &dc::Instance,
    surface: &dacite::khr_surface::SurfaceKhr
) -> Result<DeviceSettings, ()> {
    let physical_devices = instance.enumerate_physical_devices().map_err(|e| {
        println!("Failed to enumerate physical devices ({})", e);
    })?;

    for physical_device in physical_devices {
        if let Ok(device_settings) = check_device_suitability(physical_device, surface) {
            return Ok(device_settings);
        }
    }

    println!("Failed to find a suitable device");
    Err(())
}

fn create_device(
    physical_device: &dc::PhysicalDevice,
    device_extensions: dc::DeviceExtensions,
    queue_family_indices: &QueueFamilyIndices
) -> Result<dc::Device, ()> {

        let device_queue_create_infos = vec![
            dc::DeviceQueueCreateInfo {
                flags: dc::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family_indices.graphics,
                queue_priorities: vec![1.0],
                chain: None,
            },
        ];

        let device_create_info = dc::DeviceCreateInfo {
            flags: dc::DeviceCreateFlags::empty(),
            queue_create_infos: device_queue_create_infos,
            enabled_layers: vec![],
            enabled_extensions: device_extensions,
            enabled_features: None,
            chain: None,
        };

        physical_device.create_device(&device_create_info, None).map_err(|e| {
            println!("Failed to create device ({})", e);
        })
}