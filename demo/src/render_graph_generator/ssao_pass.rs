use crate::phases::PostProcessRenderPhase;
use rafx::framework::{MaterialPassResource, ResourceArc};
use rafx::graph::*;
use rafx::render_features::RenderPhase;

use super::RenderGraphContext;
use super::EMPTY_VERTEX_LAYOUT;
use rafx::api::{RafxColorClearValue, RafxFormat, RafxResourceType};

pub(super) struct SsaoPass {
    pub(super) ambient_occlusion: RenderGraphImageUsageId,
}

pub(super) fn ssao_pass(
    context: &mut RenderGraphContext,
    depth_prepass: &super::depth_prepass::DepthPrepass,
    ssao_material_pass: ResourceArc<MaterialPassResource>,
) -> SsaoPass {
    let node = context
        .graph
        .add_node("Ssao", RenderGraphQueue::DefaultGraphics);

    context.graph.read_depth_attachment(
        node,
        depth_prepass.depth,
        RenderGraphImageConstraint {
            samples: Some(context.graph_config.samples),
            format: Some(context.graph_config.depth_format),
            ..Default::default()
        },
        Default::default(),
    );

    let ambient_occlusion = context.graph.create_color_attachment(
        node,
        0, // color attachment index
        Some(RafxColorClearValue([1.0; 4])),
        RenderGraphImageConstraint {
            samples: Some(context.graph_config.samples),
            format: Some(RafxFormat::R8_UNORM),
            resource_type: RafxResourceType::TEXTURE | RafxResourceType::RENDER_TARGET_COLOR,
            extents: Some(RenderGraphImageExtents::MatchSurface),
            layer_count: Some(1),
            mip_count: Some(1),
        },
        Default::default(),
    );

    context
        .graph
        .set_image_name(ambient_occlusion, "ambient_occlusion");

    let sample_depth_image = context.graph.sample_image(
        node,
        depth_prepass.depth,
        Default::default(),
        Default::default(),
    );
    context.graph.set_image_name(depth_prepass.depth, "depth");
    let sample_normal_image = context.graph.sample_image(
        node,
        depth_prepass.normal,
        Default::default(),
        Default::default(),
    );
    context.graph.set_image_name(depth_prepass.normal, "normal");

    context.graph.set_renderpass_callback(node, move |args| {
        // Get the depth image from before
        let sample_depth_image = args.graph_context.image_view(sample_depth_image);
        let sample_normal_image = args.graph_context.image_view(sample_normal_image);

        // Get the pipeline
        let pipeline = args
            .graph_context
            .resource_context()
            .graphics_pipeline_cache()
            .get_or_create_graphics_pipeline(
                PostProcessRenderPhase::render_phase_index(),
                &ssao_material_pass,
                &args.render_target_meta,
                &EMPTY_VERTEX_LAYOUT,
            )?;

        let descriptor_set_layouts = &pipeline.get_raw().descriptor_set_layouts;

        // Set up a descriptor set pointing at the image so we can sample from it
        let mut descriptor_set_allocator = args
            .graph_context
            .resource_context()
            .create_descriptor_set_allocator();

        let kernel = [
            // precalculated hemisphere kernel (low discrepancy noiser)
            [-0.668154, -0.084296, 0.219458],
            [-0.092521, 0.141327, 0.505343],
            [-0.041960, 0.700333, 0.365754],
            [0.722389, -0.015338, 0.084357],
            [-0.815016, 0.253065, 0.465702],
            [0.018993, -0.397084, 0.136878],
            [0.617953, -0.234334, 0.513754],
            [-0.281008, -0.697906, 0.240010],
            [0.303332, -0.443484, 0.588136],
            [-0.477513, 0.559972, 0.310942],
            [0.307240, 0.076276, 0.324207],
            [-0.404343, -0.615461, 0.098425],
            [0.152483, -0.326314, 0.399277],
            [0.435708, 0.630501, 0.169620],
            [0.878907, 0.179609, 0.266964],
            [-0.049752, -0.232228, 0.264012],
            [0.537254, -0.047783, 0.693834],
            [0.001000, 0.177300, 0.096643],
            [0.626400, 0.524401, 0.492467],
            [-0.708714, -0.223893, 0.182458],
            [-0.106760, 0.020965, 0.451976],
            [-0.285181, -0.388014, 0.241756],
            [0.241154, -0.174978, 0.574671],
            [-0.405747, 0.080275, 0.055816],
            [0.079375, 0.289697, 0.348373],
            [0.298047, -0.309351, 0.114787],
            [-0.616434, -0.117369, 0.475924],
            [-0.035249, 0.134591, 0.840251],
            [0.175849, 0.971033, 0.211778],
            [0.024805, 0.348056, 0.240006],
            [-0.267123, 0.204885, 0.688595],
            [-0.077639, -0.753205, 0.070938],
        ];

        let ssao_material_dyn_set = descriptor_set_allocator.create_descriptor_set(
            &descriptor_set_layouts[shaders::ssao_frag::SSAO_CONFIG_DESCRIPTOR_SET_INDEX],
            shaders::ssao_frag::DescriptorSet1Args {
                ssao_config: &shaders::ssao_frag::SsaoConfigUboUniform {
                    kernel,
                    kernel_size: 8,
                    radius_vs: 0.2,
                    bias: 0.025,
                    ..Default::default()
                },
                depth_texture: sample_depth_image.as_ref().unwrap(),
                normal_texture: sample_normal_image.as_ref().unwrap(),
            },
        )?;

        // Explicit flush since we're going to use the descriptors immediately
        descriptor_set_allocator.flush_changes()?;

        // Draw calls
        let command_buffer = &args.command_buffer;
        command_buffer.cmd_bind_pipeline(&*pipeline.get_raw().pipeline)?;
        ssao_material_dyn_set.bind(command_buffer)?;
        command_buffer.cmd_draw(3, 0)?;

        Ok(())
    });

    return SsaoPass { ambient_occlusion };
}
