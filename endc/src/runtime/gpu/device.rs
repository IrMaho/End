use super::error::GpuError;
use wgpu::*;

#[derive(Debug)]
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub adapter_info: AdapterInfo,
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    /// Initialize a real wgpu GPU compute context (or return explicit GpuError)
    pub fn init() -> Result<Self, GpuError> {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::empty(),
            backend_options: BackendOptions::default(),
        });

        // 1. Attempt to request high-performance adapter
        let adapter_opt = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));

        let adapter = match adapter_opt {
            Some(ad) => ad,
            None => {
                // Try fallback / default adapter
                match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: true,
                })) {
                    Some(ad) => ad,
                    None => {
                        return Err(GpuError::NoAdapterAvailable(
                            "No compute-capable GPU or fallback adapter found on system".to_string(),
                        ));
                    }
                }
            }
        };

        let adapter_info = adapter.get_info();

        // 2. Request Device and Queue with maximum supported buffer limits
        let limits = adapter.limits();
        let device_desc = DeviceDescriptor {
            label: Some("End Gpu Compute Device"),
            required_features: Features::empty(),
            required_limits: limits,
            memory_hints: MemoryHints::Performance,
        };

        let (device, queue) = pollster::block_on(adapter.request_device(&device_desc, None))
            .map_err(|e| GpuError::DeviceRequestFailed(format!("Failed to request GPU device from adapter '{}': {}", adapter_info.name, e)))?;

        Ok(Self {
            instance,
            adapter,
            adapter_info,
            device,
            queue,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_info.name
    }

    pub fn backend_name(&self) -> String {
        format!("{:?}", self.adapter_info.backend)
    }

    pub fn device_type_str(&self) -> String {
        format!("{:?}", self.adapter_info.device_type)
    }

    pub fn driver_info(&self) -> &str {
        &self.adapter_info.driver
    }
}
