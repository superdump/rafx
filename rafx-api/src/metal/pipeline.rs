use crate::{RafxGraphicsPipelineDef, RafxRootSignature, RafxPipelineType, RafxComputePipelineDef, RafxResult};
use crate::metal::RafxDeviceContextMetal;

#[derive(Debug)]
pub struct RafxPipelineMetal {
    pipeline_type: RafxPipelineType,
    // It's a RafxRootSignatureMetal, but stored as RafxRootSignature so we can return refs to it
    root_signature: RafxRootSignature,
}

impl RafxPipelineMetal {
    pub fn pipeline_type(&self) -> RafxPipelineType {
        self.pipeline_type
    }

    pub fn root_signature(&self) -> &RafxRootSignature {
        &self.root_signature
    }

    pub fn new_graphics_pipeline(
        device_context: &RafxDeviceContextMetal,
        pipeline_def: &RafxGraphicsPipelineDef,
    ) -> RafxResult<Self> {
        unimplemented!();
    }

    pub fn new_compute_pipeline(
        device_context: &RafxDeviceContextMetal,
        pipeline_def: &RafxComputePipelineDef,
    ) -> RafxResult<Self> {
        unimplemented!();
    }
}