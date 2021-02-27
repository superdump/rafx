use super::write::TextCommandWriter;
use crate::features::text::{
    TextRenderFeature, TextUniformBufferObject,
    ExtractedTextData,
};
use crate::phases::OpaqueRenderPhase;
use rafx::api::RafxBufferDef;
use rafx::framework::{MaterialPassResource, ResourceArc};
use rafx::nodes::{FeatureCommandWriter, FeatureSubmitNodes, FramePacket, PrepareJob, RenderFeature, RenderFeatureIndex, RenderView, ViewSubmitNodes, RenderJobPrepareContext};

pub struct TextPrepareJobImpl {
    text_material_pass: ResourceArc<MaterialPassResource>,
    extracted_text_data: ExtractedTextData,
}

impl TextPrepareJobImpl {
    pub(super) fn new(
        text_material_pass: ResourceArc<MaterialPassResource>,
        extracted_text_data: ExtractedTextData,
    ) -> Self {
        TextPrepareJobImpl {
            text_material_pass,
            extracted_text_data,
        }
    }
}

impl<'a> PrepareJob for TextPrepareJobImpl {
    fn prepare(
        self: Box<Self>,
        prepare_context: &RenderJobPrepareContext,
        _frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> (
        Box<dyn FeatureCommandWriter>,
        FeatureSubmitNodes,
    ) {
        profiling::scope!("Text Prepare");

        let mut descriptor_set_allocator = prepare_context
            .resource_context
            .create_descriptor_set_allocator();

        // Get the layouts for both descriptor sets
        let per_view_vert_descriptor_set_layout =
            &self.text_material_pass.get_raw().descriptor_set_layouts
                [shaders::text_vert::PER_VIEW_DATA_DESCRIPTOR_SET_INDEX];
        let per_view_frag_descriptor_set_layout =
            &self.text_material_pass.get_raw().descriptor_set_layouts
                [shaders::text_frag::TEX_DESCRIPTOR_SET_INDEX];

        // Will hold the descriptor set for each view
        // temporarily separate sets due to a metal backend issue that isn't resolved yet
        let mut per_view_vert_descriptor_sets = Vec::default();
        let mut per_view_frag_descriptor_sets = Vec::default();

        let mut submit_nodes = FeatureSubmitNodes::default();

        if let Some(texture) = &self.extracted_text_data.texture {
            for view in views {
                //
                // Setup the vertex shader descriptor set
                //
                let text_view = TextUniformBufferObject {
                    view_proj: (view.projection_matrix() * view.view_matrix()).to_cols_array_2d(),
                };

                let vert_descriptor_set = descriptor_set_allocator
                    .create_descriptor_set(
                        per_view_vert_descriptor_set_layout,
                        shaders::text_vert::DescriptorSet1Args {
                            per_view_data: &text_view,
                        },
                    )
                    .unwrap();

                // Grow the array if necessary
                per_view_vert_descriptor_sets.resize(
                    per_view_vert_descriptor_sets
                        .len()
                        .max(view.view_index() as usize + 1),
                    None,
                );

                per_view_vert_descriptor_sets[view.view_index() as usize] = Some(vert_descriptor_set.clone());

                //
                // Setup the frag shader descriptor set
                //
                let frag_descriptor_set = descriptor_set_allocator
                    .create_descriptor_set(
                        per_view_frag_descriptor_set_layout,
                        shaders::text_frag::DescriptorSet0Args {
                            tex: texture
                            //per_frame_data: &text_view,
                        },
                    )
                    .unwrap();

                // Grow the array if necessary
                per_view_frag_descriptor_sets.resize(
                    per_view_frag_descriptor_sets
                        .len()
                        .max(view.view_index() as usize + 1),
                    None,
                );

                per_view_frag_descriptor_sets[view.view_index() as usize] = Some(frag_descriptor_set.clone());

                //
                // Submit a single node for each view
                // TODO: Submit separate nodes for transparency/text positioned in 3d
                //
                let mut view_submit_nodes =
                    ViewSubmitNodes::new(self.feature_index(), view.render_phase_mask());
                view_submit_nodes.add_submit_node::<OpaqueRenderPhase>(0, 0, 0.0);
                submit_nodes.add_submit_nodes_for_view(view, view_submit_nodes);
            }
        }

        //
        // Update the vertex buffer
        //
        let dyn_resource_allocator = prepare_context
            .resource_context
            .create_dyn_resource_allocator_set();

        let vertex_data = &self.extracted_text_data.vertex_data;
        println!("vertex data: {}", vertex_data.len());

        let vertex_buffer = if !vertex_data.is_empty() {
            let vertex_buffer = prepare_context
                .device_context
                .create_buffer(&RafxBufferDef::for_staging_vertex_buffer_data(&vertex_data))
                .unwrap();

            vertex_buffer
                .copy_to_host_visible_buffer(vertex_data.as_slice())
                .unwrap();

            Some(dyn_resource_allocator.insert_buffer(vertex_buffer))
        } else {
            None
        };

        let writer = Box::new(TextCommandWriter {
            vertex_count: vertex_data.len() as u32,
            vertex_buffer,
            text_material_pass: self.text_material_pass,
            per_view_vert_descriptor_sets,
            per_view_frag_descriptor_sets,
            texture: self.extracted_text_data.texture,
            image_update: self.extracted_text_data.image_update,
        });

        (writer, submit_nodes)
    }

    fn feature_debug_name(&self) -> &'static str {
        TextRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        TextRenderFeature::feature_index()
    }
}
