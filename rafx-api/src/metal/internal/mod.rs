use crate::{RafxMemoryUsage, RafxFilterType, RafxMipMapMode, RafxCompareOp};
use metal::{MTLResourceOptions, MTLCPUCacheMode, MTLStorageMode, MTLSamplerMinMagFilter, MTLSamplerMipFilter, MTLCompareFunction};

pub mod util;

impl Into<MTLSamplerMinMagFilter> for RafxFilterType {
    fn into(self) -> MTLSamplerMinMagFilter {
        match self {
            RafxFilterType::Nearest => MTLSamplerMinMagFilter::Nearest,
            RafxFilterType::Linear => MTLSamplerMinMagFilter::Linear,
        }
    }
}

impl Into<MTLSamplerMipFilter> for RafxMipMapMode {
    fn into(self) -> MTLSamplerMipFilter {
        match self {
            RafxMipMapMode::Nearest => MTLSamplerMipFilter::Nearest,
            RafxMipMapMode::Linear => MTLSamplerMipFilter::Linear,
        }
    }
}

impl Into<MTLCompareFunction> for RafxCompareOp {
    fn into(self) -> MTLCompareFunction {
        match self {
            RafxCompareOp::Never => MTLCompareFunction::Never,
            RafxCompareOp::Less => MTLCompareFunction::Less,
            RafxCompareOp::Equal => MTLCompareFunction::Equal,
            RafxCompareOp::LessOrEqual => MTLCompareFunction::LessEqual,
            RafxCompareOp::Greater => MTLCompareFunction::Greater,
            RafxCompareOp::NotEqual => MTLCompareFunction::NotEqual,
            RafxCompareOp::GreaterOrEqual => MTLCompareFunction::GreaterEqual,
            RafxCompareOp::Always => MTLCompareFunction::Always,
        }
    }
}

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

