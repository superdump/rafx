use crate::{RafxMemoryUsage, RafxFilterType, RafxMipMapMode, RafxCompareOp, RafxSampleCount, RafxVertexAttributeRate, RafxPrimitiveTopology, RafxBlendOp, RafxBlendFactor};
use metal::{MTLResourceOptions, MTLCPUCacheMode, MTLStorageMode, MTLSamplerMinMagFilter, MTLSamplerMipFilter, MTLCompareFunction, MTLStepFunction, MTLPrimitiveTopologyClass, MTLBlendOperation, MTLBlendFactor, MTLVertexStepFunction};
use cocoa_foundation::foundation::NSUInteger;

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

impl Into<NSUInteger> for RafxSampleCount {
    fn into(self) -> NSUInteger {
        match self {
            RafxSampleCount::SampleCount1 => 1,
            RafxSampleCount::SampleCount2 => 2,
            RafxSampleCount::SampleCount4 => 4,
            RafxSampleCount::SampleCount8 => 8,
            RafxSampleCount::SampleCount16 => 16,
        }
    }
}

impl Into<MTLVertexStepFunction> for RafxVertexAttributeRate {
    fn into(self) -> MTLVertexStepFunction {
        match self {
            RafxVertexAttributeRate::Vertex => MTLVertexStepFunction::PerVertex,
            RafxVertexAttributeRate::Instance => MTLVertexStepFunction::PerInstance,
        }
    }
}

impl Into<MTLPrimitiveTopologyClass> for RafxPrimitiveTopology {
    fn into(self) -> MTLPrimitiveTopologyClass {
        match self {
            RafxPrimitiveTopology::PointList => MTLPrimitiveTopologyClass::Point,
            RafxPrimitiveTopology::LineList => MTLPrimitiveTopologyClass::Line,
            RafxPrimitiveTopology::LineStrip => MTLPrimitiveTopologyClass::Line,
            RafxPrimitiveTopology::TriangleList => MTLPrimitiveTopologyClass::Triangle,
            RafxPrimitiveTopology::TriangleStrip => MTLPrimitiveTopologyClass::Triangle,
            RafxPrimitiveTopology::PatchList => MTLPrimitiveTopologyClass::Triangle,
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

impl Into<MTLBlendOperation> for RafxBlendOp {
    fn into(self) -> MTLBlendOperation {
        match self {
            RafxBlendOp::Add => MTLBlendOperation::Add,
            RafxBlendOp::Subtract => MTLBlendOperation::Subtract,
            RafxBlendOp::ReverseSubtract => MTLBlendOperation::ReverseSubtract,
            RafxBlendOp::Min => MTLBlendOperation::Min,
            RafxBlendOp::Max => MTLBlendOperation::Max,
        }
    }
}

impl Into<MTLBlendFactor> for RafxBlendFactor {
    fn into(self) -> MTLBlendFactor {
        match self {
            RafxBlendFactor::Zero => MTLBlendFactor::Zero,
            RafxBlendFactor::One => MTLBlendFactor::One,
            RafxBlendFactor::SrcColor => MTLBlendFactor::SourceColor,
            RafxBlendFactor::OneMinusSrcColor => MTLBlendFactor::OneMinusSourceColor,
            RafxBlendFactor::DstColor => MTLBlendFactor::DestinationColor,
            RafxBlendFactor::OneMinusDstColor => MTLBlendFactor::OneMinusDestinationColor,
            RafxBlendFactor::SrcAlpha => MTLBlendFactor::SourceAlpha,
            RafxBlendFactor::OneMinusSrcAlpha => MTLBlendFactor::OneMinusSourceAlpha,
            RafxBlendFactor::DstAlpha => MTLBlendFactor::DestinationAlpha,
            RafxBlendFactor::OneMinusDstAlpha => MTLBlendFactor::OneMinusDestinationAlpha,
            RafxBlendFactor::SrcAlphaSaturate => MTLBlendFactor::SourceAlphaSaturated,
            RafxBlendFactor::ConstantColor => MTLBlendFactor::BlendColor,
            RafxBlendFactor::OneMinusConstantColor => MTLBlendFactor::OneMinusBlendColor,
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

