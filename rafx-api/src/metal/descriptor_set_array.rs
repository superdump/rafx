use crate::{RafxRootSignature, RafxDescriptorUpdate, RafxResult, RafxDescriptorSetArrayDef};
use crate::metal::RafxDeviceContextMetal;

struct DescriptorUpdateData {
    // one per set * elements in each descriptor
    buffers: Vec<metal::Buffer>,
    textures: Vec<metal::Texture>,
    samplers: Vec<metal::SamplerState>,
    update_data_count: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct RafxDescriptorSetHandleMetal;

pub struct RafxDescriptorSetArrayMetal {
    root_signature: RafxRootSignature,
    set_index: u32,
}

impl RafxDescriptorSetArrayMetal {
    pub fn root_signature(&self) -> &RafxRootSignature {
        &self.root_signature
    }

    pub fn set_index(&self) -> u32 {
        self.set_index
    }

    pub fn handle(
        &self,
        index: u32,
    ) -> Option<RafxDescriptorSetHandleMetal> {
        unimplemented!();
    }

    pub(crate) fn new(
        device_context: &RafxDeviceContextMetal,
        descriptor_set_array_def: &RafxDescriptorSetArrayDef,
    ) -> RafxResult<Self> {
        let root_signature = descriptor_set_array_def
            .root_signature
            .metal_root_signature()
            .unwrap()
            .clone();

        let layout_index = descriptor_set_array_def.set_index as usize;
        let update_data_count = descriptor_set_array_def.array_length
            * root_signature.inner.layouts[layout_index].update_data_count_per_set as usize;


    }

    pub fn update_descriptor_set(
        &mut self,
        descriptor_updates: &[RafxDescriptorUpdate],
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn flush_descriptor_set_updates(&mut self) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn queue_descriptor_set_update(
        &mut self,
        update: &RafxDescriptorUpdate,
    ) -> RafxResult<()> {
        unimplemented!();
    }
}

impl std::fmt::Debug for RafxDescriptorSetArrayMetal {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("RafxDescriptorSetArrayMetal")
            //.field("first_descriptor_set", &self.descriptor_sets[0])
            //.field("root_signature", &self.root_signature)
            //.field("set_index", &self.set_index)
            //.field("pending_write_count", &self.pending_writes.len())
            .finish()
    }
}