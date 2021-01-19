use crate::metal::RafxDeviceContextMetal;
use crate::{RafxRootSignatureDef, RafxResult, RafxResourceType, RafxSampler, RafxDescriptorIndex, RafxPipelineType, MAX_DESCRIPTOR_SET_LAYOUTS};
use std::sync::Arc;
use fnv::FnvHashMap;

//TODO: Could compact this down quite a bit
#[derive(Clone, Debug)]
pub(crate) struct DescriptorInfo {
    pub(crate) name: Option<String>,
    pub(crate) resource_type: RafxResourceType,

    // Also the set layout
    pub(crate) set_index: u32,
    // Binding within the set
    pub(crate) binding: u32,
    // Used for arrays of textures, samplers, etc.
    pub(crate) element_count: u32,
    // Index into DescriptorSetLayoutInfo::descriptors list
    // NOT THE BINDING INDEX!!!
    pub(crate) descriptor_index: RafxDescriptorIndex,

    pub(crate) immutable_sampler: Option<RafxSampler>,
}

#[derive(Default, Debug)]
pub(crate) struct DescriptorSetLayoutInfo {
    // Settable descriptors, immutable samplers are omitted
    pub(crate) descriptors: Vec<RafxDescriptorIndex>,

    // Indexes binding index to the descriptors list
    pub(crate) binding_to_descriptor_index: FnvHashMap<u32, RafxDescriptorIndex>,

    sampler_count: u32,
    texture_count: u32,
    buffer_count: u32,
}

#[derive(Debug)]
struct RafxRootSignatureMetalInner {
    pub(crate) device_context: RafxDeviceContextMetal,
    pub(crate) pipeline_type: RafxPipelineType,
    pub(crate) descriptors: Vec<DescriptorInfo>,
    pub(crate) name_to_descriptor_index: FnvHashMap<String, RafxDescriptorIndex>,
    // Keeps them in scope so they don't drop
    //TODO: Can potentially remove, they are held in DescriptorInfo too
    immutable_samplers: Vec<RafxSampler>,
}

#[derive(Clone, Debug)]
pub struct RafxRootSignatureMetal {
    inner: Arc<RafxRootSignatureMetalInner>
}

impl RafxRootSignatureMetal {
    pub fn device_context(&self) -> &RafxDeviceContextMetal {
        &self.inner.device_context
    }

    pub fn pipeline_type(&self) -> RafxPipelineType {
        self.inner.pipeline_type
    }

    pub fn find_descriptor_by_name(
        &self,
        name: &str,
    ) -> Option<RafxDescriptorIndex> {
        self.inner.name_to_descriptor_index.get(name).copied()
    }

    // pub fn find_descriptor_by_binding(
    //     &self,
    //     set_index: u32,
    //     binding: u32,
    // ) -> Option<RafxDescriptorIndex> {
    //     self.inner
    //         .layouts
    //         .get(set_index as usize)
    //         .and_then(|x| x.binding_to_descriptor_index.get(&binding))
    //         .copied()
    // }

    pub(crate) fn descriptor(
        &self,
        descriptor_index: RafxDescriptorIndex,
    ) -> Option<&DescriptorInfo> {
        self.inner.descriptors.get(descriptor_index.0 as usize)
    }

    pub fn new(
        device_context: &RafxDeviceContextMetal,
        root_signature_def: &RafxRootSignatureDef
    ) -> RafxResult<Self> {
        log::trace!("Create RafxRootSignatureMetal");

        let mut immutable_samplers = vec![];
        for sampler_list in root_signature_def.immutable_samplers {
            for sampler in sampler_list.samplers {
                immutable_samplers.push(sampler.clone());
            }
        }

        // Make sure all shaders are compatible/build lookup of shared data from them
        let (pipeline_type, merged_resources, _merged_resources_name_index_map) =
            crate::internal_shared::merge_resources(root_signature_def)?;

        let mut sampler_count = 0;
        let mut texture_count = 0;
        let mut buffer_count = 0;

        let mut descriptors = Vec::with_capacity(merged_resources.len());
        let mut name_to_descriptor_index = FnvHashMap::default();

        for resource in &merged_resources {
            if resource.resource_type.intersects(RafxResourceType::SAMPLER) {
                sampler_count += 1;
            } else if resource.resource_type.intersects(RafxResourceType::TEXTURE | RafxResourceType::TEXTURE_READ_WRITE) {
                texture_count += 1;
            } else {
                buffer_count += 1;
            }

            if resource.set_index as usize >= MAX_DESCRIPTOR_SET_LAYOUTS {
                Err(format!(
                    "Descriptor (set={:?} binding={:?}) named {:?} has a set index >= 4. This is not supported",
                    resource.set_index, resource.binding, resource.name,
                ))?;
            }

            let immutable_sampler = crate::internal_shared::find_immutable_sampler_index(
                root_signature_def.immutable_samplers,
                &resource.name,
                resource.set_index,
                resource.binding,
            );

            let descriptor_index = RafxDescriptorIndex(descriptors.len() as u32);

            // Add it to the descriptor list
            descriptors.push(DescriptorInfo {
                name: resource.name.clone(),
                resource_type: resource.resource_type,
                //texture_dimensions: resource.texture_dimensions,
                set_index: resource.set_index,
                binding: resource.binding,
                element_count: resource.element_count_normalized(),
                descriptor_index,
                immutable_sampler: immutable_sampler.map(|x| immutable_samplers[x].clone()),
            });

            if let Some(name) = resource.name.as_ref() {
                name_to_descriptor_index.insert(name.clone(), descriptor_index);
            }
        }

        let inner = RafxRootSignatureMetalInner {
            device_context: device_context.clone(),
            pipeline_type,
            descriptors,
            name_to_descriptor_index,
            immutable_samplers,
            texture_count,
            sampler_count,
            buffer_count,
        };

        Ok(RafxRootSignatureMetal {
            inner: Arc::new(inner),
        })
    }
}