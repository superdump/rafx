use rafx_api::{RafxSamplerDef, RafxShaderResource, RafxShaderStageReflection};
use crate::DescriptorSetLayoutBinding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ReflectedDescriptorSetLayoutBinding {
    // Basic info required to create the RafxRootSignature
    pub resource: RafxShaderResource,

    // Samplers created here will be automatically created/bound
    pub immutable_samplers: Option<Vec<RafxSamplerDef>>,

    // If this is non-zero we will allocate a buffer owned by the descriptor set pool chunk,
    // and automatically bind it - this makes binding data easy to do without having to manage
    // buffers.
    pub internal_buffer_per_descriptor_size: Option<u32>,
}

impl Into<DescriptorSetLayoutBinding> for ReflectedDescriptorSetLayoutBinding {
    fn into(self) -> DescriptorSetLayoutBinding {
        DescriptorSetLayoutBinding {
            resource: self.resource.clone(),
            immutable_samplers: self.immutable_samplers.clone(),
            internal_buffer_per_descriptor_size: self.internal_buffer_per_descriptor_size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReflectedDescriptorSetLayout {
    // These are NOT indexable by binding (i.e. may be sparse)
    pub bindings: Vec<ReflectedDescriptorSetLayoutBinding>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReflectedVertexInput {
    pub name: String,
    pub semantic: String,
    pub location: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReflectedEntryPoint {
    // The reflection data used by rafx API
    pub rafx_reflection: RafxShaderStageReflection,

    // Additional reflection data used by the framework level for descriptor sets
    pub descriptor_set_layouts: Vec<Option<ReflectedDescriptorSetLayout>>,

    // Additional reflection data used by the framework level for vertex inputs
    pub vertex_inputs: Vec<ReflectedVertexInput>,
}

// An import format that will get turned into ShaderAssetData
#[derive(Serialize, Deserialize)]
pub struct CookedShader {
    #[serde(with = "serde_bytes")]
    pub spv: Vec<u8>,
    //TODO: We ideally package binary but this is only possible with apple shader tools installed,
    // which is only available on win/mac. So we'll want a fallback path so that it's not impossible
    // to produce a cooked shader on machines without the tools. (Also the tools don't provide an
    // API so will need to figure out how to compile the shader programmatically.)
    pub metal_source: String,
    pub entry_points: Vec<ReflectedEntryPoint>,
}