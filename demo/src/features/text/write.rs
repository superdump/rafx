use crate::features::text::{TextDrawCall, TextRenderFeature, TextImageUpdate};
use crate::render_contexts::RenderJobWriteContext;
use rafx::api::{RafxResult, RafxVertexBufferBinding, RafxTextureBarrier, RafxResourceState};
use rafx::framework::{BufferResource, DescriptorSetArc, MaterialPassResource, ResourceArc, ImageViewResource};
use rafx::nodes::{
    FeatureCommandWriter, RenderFeature, RenderFeatureIndex, RenderPhaseIndex, RenderView,
    SubmitNodeId,
};

pub struct TextCommandWriter {
    pub(super) vertex_count: u32,
    pub(super) vertex_buffer: Option<ResourceArc<BufferResource>>,
    pub(super) text_material_pass: ResourceArc<MaterialPassResource>,
    pub(super) per_view_vert_descriptor_sets: Vec<Option<DescriptorSetArc>>,
    pub(super) per_view_frag_descriptor_sets: Vec<Option<DescriptorSetArc>>,
    pub(super) texture: Option<ResourceArc<ImageViewResource>>,
    pub(super) image_update: Option<TextImageUpdate>,
}

impl FeatureCommandWriter<RenderJobWriteContext> for TextCommandWriter {
    fn on_phase_begin(
        &self,
        write_context: &mut RenderJobWriteContext,
        view: &RenderView,
        render_phase_index: RenderPhaseIndex,
    ) -> RafxResult<()> {

        if let Some(image_update) = &self.image_update {
            write_context.command_buffer.cmd_resource_barrier(&[], &[
                RafxTextureBarrier::state_transition(
                    &self.texture.as_ref().unwrap().get_raw().image.get_raw().image,
                    RafxResourceState::SHADER_RESOURCE,
                    RafxResourceState::COPY_DST
                )
            ])?;

            // copy buffer to texture

            write_context.command_buffer.cmd_resource_barrier(&[], &[
                RafxTextureBarrier::state_transition(
                    &self.texture.as_ref().unwrap().get_raw().image.get_raw().image,
                    RafxResourceState::COPY_DST,
                    RafxResourceState::SHADER_RESOURCE
                )
            ])?;
        }


        Ok(())
    }

    fn apply_setup(
        &self,
        write_context: &mut RenderJobWriteContext,
        view: &RenderView,
        render_phase_index: RenderPhaseIndex,
    ) -> RafxResult<()> {
        if let Some(vertex_buffer) = self.vertex_buffer.as_ref() {
            let pipeline = write_context
                .resource_context
                .graphics_pipeline_cache()
                .get_or_create_graphics_pipeline(
                    render_phase_index,
                    &self.text_material_pass,
                    &write_context.render_target_meta,
                    &*super::TEXT_VERTEX_LAYOUT,
                )?;

            let command_buffer = &write_context.command_buffer;
            command_buffer.cmd_bind_pipeline(&*pipeline.get_raw().pipeline)?;

            // if self.per_view_vert_descriptor_sets[view.view_index() as usize].is_none() ||
            //     self.per_view_frag_descriptor_sets[view.view_index() as usize].is_none() {
            //     return;
            // }

            self.per_view_vert_descriptor_sets[view.view_index() as usize]
                .as_ref()
                .unwrap()
                .bind(command_buffer)?;

            self.per_view_frag_descriptor_sets[view.view_index() as usize]
                .as_ref()
                .unwrap()
                .bind(command_buffer)?;

            command_buffer.cmd_bind_vertex_buffers(
                0,
                &[RafxVertexBufferBinding {
                    buffer: &*vertex_buffer.get_raw().buffer,
                    offset: 0,
                }],
            )?;
        }
        Ok(())
    }

    fn render_element(
        &self,
        write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
        index: SubmitNodeId,
    ) -> RafxResult<()> {
        // The prepare phase emits a single node which will draw everything. In the future it might
        // emit a node per draw call that uses transparency
        if index == 0 {
            if let Some(vertex_buffer) = &self.vertex_buffer {
                let command_buffer = &write_context.command_buffer;
                command_buffer.cmd_draw_instanced(self.vertex_count, 0, 4, 0)?;
            }
        }
        Ok(())
    }

    fn feature_debug_name(&self) -> &'static str {
        TextRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        TextRenderFeature::feature_index()
    }
}
