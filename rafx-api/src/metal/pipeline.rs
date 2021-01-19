use crate::{RafxGraphicsPipelineDef, RafxRootSignature, RafxPipelineType, RafxComputePipelineDef, RafxResult, RafxShaderStageFlags};
use crate::metal::RafxDeviceContextMetal;

#[derive(Debug)]
enum MetalPipelineState {
    Graphics(metal::RenderPipelineState),
    Compute(metal::ComputePipelineState),
}

#[derive(Debug)]
pub struct RafxPipelineMetal {
    pipeline_type: RafxPipelineType,
    // It's a RafxRootSignatureMetal, but stored as RafxRootSignature so we can return refs to it
    root_signature: RafxRootSignature,
    pipeline: MetalPipelineState,
}

impl RafxPipelineMetal {
    pub fn pipeline_type(&self) -> RafxPipelineType {
        self.pipeline_type
    }

    pub fn root_signature(&self) -> &RafxRootSignature {
        &self.root_signature
    }

    pub fn metal_render_pipeline(&self) -> Option<&metal::RenderPipelineStateRef> {
        match &self.pipeline {
            MetalPipelineState::Graphics(pipeline) => Some(pipeline.as_ref()),
            MetalPipelineState::Compute(_) => None,
        }
    }

    pub fn metal_compute_pipeline(&self) -> Option<&metal::ComputePipelineStateRef> {
        match &self.pipeline {
            MetalPipelineState::Graphics(_) => None,
            MetalPipelineState::Compute(pipeline) => Some(pipeline.as_ref()),
        }
    }

    pub fn new_graphics_pipeline(
        device_context: &RafxDeviceContextMetal,
        pipeline_def: &RafxGraphicsPipelineDef,
    ) -> RafxResult<Self> {
        let mut pipeline = metal::RenderPipelineDescriptor::new();

        let mut vertex_function = None;
        let mut fragment_function = None;

        for stage in pipeline_def.shader.metal_shader().unwrap().stages() {
            if stage.shader_stage.intersects(RafxShaderStageFlags::VERTEX) {
                assert!(vertex_function.is_none());
                vertex_function = Some(stage.shader_module.metal_shader_module().unwrap().library().get_function(
                    &stage.entry_point,
                    None
                )?);
            }

            if stage.shader_stage.intersects(RafxShaderStageFlags::FRAGMENT) {
                assert!(fragment_function.is_none());
                fragment_function = Some(stage.shader_module.metal_shader_module().unwrap().library().get_function(
                    &stage.entry_point,
                    None
                )?);
            }
        }

        let vertex_function = vertex_function.ok_or("Could not find vertex function")?;
        let fragment_function = fragment_function.ok_or("Could not find fragment function")?;

        pipeline.set_vertex_function(Some(vertex_function.as_ref()));
        pipeline.set_fragment_function(Some(fragment_function.as_ref()));
        pipeline.set_sample_count(pipeline_def.sample_count.into());

        let mut vertex_descriptor = metal::VertexDescriptor::new();
        for attribute in &pipeline_def.vertex_layout.attributes {
            let mut attribute_descriptor = vertex_descriptor.attributes().object_at(attribute.location as _).unwrap();
            attribute_descriptor.set_buffer_index(attribute.buffer_index as _);
            attribute_descriptor.set_format(attribute.format.into());
            attribute_descriptor.set_offset(attribute.offset as _);
        }

        for (index, binding) in pipeline_def.vertex_layout.buffers.iter().enumerate() {
            let mut layout_descriptor = vertex_descriptor.layouts().object_at(index as _).unwrap();
            layout_descriptor.set_stride(binding.stride as _);
            layout_descriptor.set_step_function(binding.rate.into());
            layout_descriptor.set_step_rate(1);
        }

        pipeline.set_input_primitive_topology(pipeline_def.primitive_topology.into());

        //TODO: Pass in number of color attachments?
        super::util::blend_def_to_attachment(pipeline_def.blend_state, &mut pipeline.color_attachments(), pipeline_def.color_formats.len());

        for (index, &color_format) in pipeline_def.color_formats.iter().enumerate() {
            pipeline.color_attachments().object_at(index as _).unwrap().set_pixel_format(color_format.into());
        }

        // cache rasterizer state:
        // cull mode, fill mode, depth bias, slope scale, clip mode, primitive type (scissor enable, multisample enable if supported by metal), winding
        // depth state

        if let Some(depth_format) = pipeline_def.depth_stencil_format {
            if depth_format.has_depth() {
                pipeline.set_depth_attachment_pixel_format(depth_format.into());
            }

            if depth_format.has_stencil() {
                pipeline.set_stencil_attachment_pixel_format(depth_format.into());
            }
        }

        let pipeline = device_context.device().new_render_pipeline_state(pipeline.as_ref())?;

        Ok(RafxPipelineMetal {
            root_signature: pipeline_def.root_signature.clone(),
            pipeline_type: pipeline_def.root_signature.pipeline_type(),
            pipeline: MetalPipelineState::Graphics(pipeline)
        })
    }

    pub fn new_compute_pipeline(
        device_context: &RafxDeviceContextMetal,
        pipeline_def: &RafxComputePipelineDef,
    ) -> RafxResult<Self> {
        unimplemented!();
    }
}