use crate::RafxMemoryUsage;
use metal::{MTLResourceOptions, MTLCPUCacheMode, MTLStorageMode};

impl RafxMemoryUsage {
    pub fn resource_options(self) -> MTLResourceOptions {
        match self {
            RafxMemoryUsage::Unknown => MTLResourceOptions::empty(),
            // TODO: This can be shared on iGPU/iOS/M1
            RafxMemoryUsage::GpuOnly => MTLResourceOptions::StorageModePrivate,
            RafxMemoryUsage::CpuOnly => MTLResourceOptions::StorageModeShared | MTLResourceOptions::CPUCacheModeDefaultCache,
            RafxMemoryUsage::CpuToGpu => MTLResourceOptions::StorageModeShared | MTLResourceOptions::CPUCacheModeWriteCombined,
            RafxMemoryUsage::GpuToCpu => MTLResourceOptions::StorageModeShared | MTLResourceOptions::CPUCacheModeDefaultCache,
        }
    }

    pub fn cpu_cache_mode(self) -> MTLCPUCacheMode {
        match self {
            RafxMemoryUsage::Unknown => MTLCPUCacheMode::DefaultCache,
            RafxMemoryUsage::GpuOnly => MTLCPUCacheMode::DefaultCache,
            RafxMemoryUsage::CpuOnly => MTLCPUCacheMode::DefaultCache,
            RafxMemoryUsage::CpuToGpu => MTLCPUCacheMode::WriteCombined,
            RafxMemoryUsage::GpuToCpu => MTLCPUCacheMode::DefaultCache,
        }
    }

    pub fn storage_mode(self) -> MTLStorageMode {
        match self {
            RafxMemoryUsage::Unknown => MTLStorageMode::Private,
            // TODO: This can be shared on iGPU/iOS/M1
            RafxMemoryUsage::GpuOnly => MTLStorageMode::Private,
            RafxMemoryUsage::CpuOnly => MTLStorageMode::Shared,
            RafxMemoryUsage::CpuToGpu => MTLStorageMode::Shared,
            RafxMemoryUsage::GpuToCpu => MTLStorageMode::Shared,
        }
    }
}