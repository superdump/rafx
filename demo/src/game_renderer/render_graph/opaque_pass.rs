use crate::phases::OpaqueRenderPhase;
use crate::render_contexts::RenderJobWriteContext;
use rafx::graph::*;

use super::RenderGraphContext;
use super::ShadowMapImageResources;
use rafx::api::{RafxColorClearValue, RafxDepthStencilClearValue, RafxVertexBufferBinding, RafxIndexBufferBinding, RafxIndexType};
use rafx::framework::{MaterialPassResource, ResourceArc, ImageViewResource, BufferResource};
use crate::features::skybox::SKYBOX_VERTEX_LAYOUT;
use rafx::nodes::RenderPhase;

pub(super) struct OpaquePass {
    pub(super) node: RenderGraphNodeId,
    pub(super) color: RenderGraphImageUsageId,
    pub(super) depth: RenderGraphImageUsageId,
    pub(super) shadow_maps: Vec<RenderGraphImageUsageId>,
}

pub(super) fn opaque_pass(
    context: &mut RenderGraphContext,
    skybox_material: ResourceArc<MaterialPassResource>,
    skybox_texture: ResourceArc<ImageViewResource>,
    skybox_vertex_buffer: ResourceArc<BufferResource>,
    skybox_index_buffer: ResourceArc<BufferResource>,
    shadow_map_passes: &[ShadowMapImageResources],
) -> OpaquePass {
    let node = context
        .graph
        .add_node("Opaque", RenderGraphQueue::DefaultGraphics);

    let color = context.graph.create_color_attachment(
        node,
        0,
        Some(RafxColorClearValue([0.0, 0.0, 0.0, 0.0])),
        RenderGraphImageConstraint {
            samples: Some(context.graph_config.samples),
            format: Some(context.graph_config.color_format),
            ..Default::default()
        },
        Default::default(),
    );
    context.graph.set_image_name(color, "color");

    let depth = context.graph.create_depth_attachment(
        node,
        Some(RafxDepthStencilClearValue {
            depth: 0.0,
            stencil: 0,
        }),
        RenderGraphImageConstraint {
            samples: Some(context.graph_config.samples),
            format: Some(context.graph_config.depth_format),
            ..Default::default()
        },
        Default::default(),
    );
    context.graph.set_image_name(depth, "depth");

    let mut shadow_maps = Vec::with_capacity(shadow_map_passes.len());
    for shadow_map_pass in shadow_map_passes {
        let sampled_image = match shadow_map_pass {
            ShadowMapImageResources::Single(image) => {
                context
                    .graph
                    .sample_image(node, *image, Default::default(), Default::default())
            }
            ShadowMapImageResources::Cube(cube_map_image) => context.graph.sample_image(
                node,
                *cube_map_image,
                Default::default(),
                Default::default(),
            ),
        };
        shadow_maps.push(sampled_image);
    }

    context
        .graph_callbacks
        .add_render_phase_dependency::<OpaqueRenderPhase>(node);

    let main_view = context.main_view.clone();
    context
        .graph_callbacks
        .set_renderpass_callback(node, move |args, user_context| {
            let mut write_context = RenderJobWriteContext::from_graph_visit_render_pass_args(&args);
            // user_context
            //     .prepared_render_data
            //     .write_view_phase::<OpaqueRenderPhase>(&main_view, &mut write_context)?;

            //
            // render the skybox last
            //

            // Get the pipeline
            let pipeline = args
                .graph_context
                .resource_context()
                .graphics_pipeline_cache()
                .get_or_create_graphics_pipeline(
                    OpaqueRenderPhase::render_phase_index(),
                    &skybox_material,
                    &args.render_target_meta,
                    &SKYBOX_VERTEX_LAYOUT,
                )?;

            // Set up a descriptor set pointing at the image so we can sample from it
            let mut descriptor_set_allocator = args
                .graph_context
                .resource_context()
                .create_descriptor_set_allocator();

            let mvp = (main_view.projection_matrix() * main_view.view_matrix());

            let descriptor_set_layouts = &pipeline.get_raw().descriptor_set_layouts;
            let skybox_material_dyn_set = descriptor_set_allocator.create_descriptor_set(
                &descriptor_set_layouts[shaders::skybox_frag::SKYBOX_TEX_DESCRIPTOR_SET_INDEX],
                shaders::skybox_frag::DescriptorSet0Args {
                    uniform_buffer: &shaders::skybox_frag::ArgsUniform {
                        mvp: mvp.to_cols_array_2d()
                    },
                    skybox_tex: &skybox_texture,
                },
            )?;

            // Explicit flush since we're going to use the descriptors immediately
            descriptor_set_allocator.flush_changes()?;

            // Draw calls
            let command_buffer = &args.command_buffer;
            command_buffer.cmd_bind_pipeline(&*pipeline.get_raw().pipeline)?;
            skybox_material_dyn_set.bind(command_buffer)?;
            command_buffer.cmd_bind_vertex_buffers(
                0,
                &[
                    RafxVertexBufferBinding {
                        buffer: &skybox_vertex_buffer.get_raw().buffer,
                        offset: 0
                    }
                ]
            )?;

            command_buffer.cmd_bind_index_buffer(&RafxIndexBufferBinding {
                buffer: &skybox_index_buffer.get_raw().buffer,
                index_type: RafxIndexType::Uint16,
                offset: 0
            })?;

            command_buffer.cmd_draw(crate::features::skybox::SKYBOX_CUBE_INDEX_BUFFER_DATA.len() as u32, 0)?;

            Ok(())
        });

    OpaquePass {
        node,
        color,
        depth,
        shadow_maps,
    }
}
