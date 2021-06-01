use crate::phases::PostProcessRenderPhase;
use rafx::framework::{MaterialPassResource, ResourceArc};
use rafx::graph::*;
use rafx::render_features::RenderPhase;

use super::RenderGraphContext;
use super::EMPTY_VERTEX_LAYOUT;
use rafx::api::RafxSampleCount;

pub(super) struct SsaoPass {
    pub(super) color: RenderGraphImageUsageId,
    pub(super) depth: RenderGraphImageUsageId,
    pub(super) normal: RenderGraphImageUsageId,
    pub(super) noise: RenderGraphImageUsageId,
}

pub(super) fn ssao_pass(
    context: &mut RenderGraphContext,
    depth_prepass: RenderGraphImageUsageId,
    normal_prepass: RenderGraphImageUsageId,

    bloom_blur_material_pass: ResourceArc<MaterialPassResource>,
    bloom_extract_pass: &super::BloomExtractPass,
) -> SsaoPass {
    let node = context
        .graph
        .add_node("Ssao", RenderGraphQueue::DefaultGraphics);

    context.graph.read_depth_attachment(
        node,
        depth_prepass,
        RenderGraphImageConstraint {
            samples: Some(context.graph_config.samples),
            format: Some(context.graph_config.depth_format),
            ..Default::default()
        },
        Default::default(),
    );
    context.graph.read_color_attachment(
        node,
        normal_prepass,
        RenderGraphImageConstraint {
            samples: Some(context.graph_config.samples),
            format: Some(RafxFormat::R8G8B8A8_UNORM),
            ..Default::default()
        },
        Default::default(),
    );

    let ssao_dst = context.graph.create_color_attachment(
        node,
        0,
        Some(Default::default()),
        RenderGraphImageConstraint {
            samples: Some(RafxSampleCount::SampleCount1),
            format: Some(RafxFormat::R8_UNORM),
            ..Default::default()
        },
        Default::default(),
    );
    context.graph.set_image_name(ssao_dst, "ssao_dst");

    let sample_depth_image =
        context
            .graph
            .sample_image(node, depth_prepass, Default::default(), Default::default());
    context.graph.set_image_name(depth_prepass, "depth_prepass");
    let sample_normal_image =
        context
            .graph
            .sample_image(node, normal_prepass, Default::default(), Default::default());
    context.graph.set_image_name(normal_prepass, "normal_prepass");

    let bloom_blur_material_pass = bloom_blur_material_pass.clone();
    context.graph.set_renderpass_callback(node, move |args| {
        // Get the color image from before
        let sample_image = args.graph_context.image_view(sample_image);

        // Get the pipeline
        let pipeline = args
            .graph_context
            .resource_context()
            .graphics_pipeline_cache()
            .get_or_create_graphics_pipeline(
                PostProcessRenderPhase::render_phase_index(),
                &bloom_blur_material_pass,
                &args.render_target_meta,
                &EMPTY_VERTEX_LAYOUT,
            )?;

        let descriptor_set_layouts = &pipeline.get_raw().descriptor_set_layouts;

        // Set up a descriptor set pointing at the image so we can sample from it
        let mut descriptor_set_allocator = args
            .graph_context
            .resource_context()
            .create_descriptor_set_allocator();

        let horizontal = if blur_direction == BlurDirection::Horizontal {
            1
        } else {
            0
        };

        let bloom_blur_material_dyn_set = descriptor_set_allocator.create_descriptor_set(
            &descriptor_set_layouts[shaders::bloom_blur_frag::TEX_DESCRIPTOR_SET_INDEX],
            shaders::bloom_blur_frag::DescriptorSet0Args {
                tex: sample_image.as_ref().unwrap(),
                config: &shaders::bloom_blur_frag::ConfigUniform {
                    horizontal,
                    ..Default::default()
                },
            },
        )?;

        // Explicit flush since we're going to use the descriptors immediately
        descriptor_set_allocator.flush_changes()?;

        // Draw calls
        let command_buffer = &args.command_buffer;
        command_buffer.cmd_bind_pipeline(&*pipeline.get_raw().pipeline)?;
        bloom_blur_material_dyn_set.bind(command_buffer)?;
        command_buffer.cmd_draw(3, 0)?;

        Ok(())
    });

    return SsaoPass { color: blur_src };
}

fn bloom_blur_internal_pass(
    context: &mut RenderGraphContext,
    bloom_blur_material_pass: &ResourceArc<MaterialPassResource>,
    blur_src: RenderGraphImageUsageId,
    blur_direction: BlurDirection,
) -> RenderGraphImageUsageId {
    let node = context
        .graph
        .add_node("BloomBlur", RenderGraphQueue::DefaultGraphics);
    let blur_dst = context.graph.create_color_attachment(
        node,
        0,
        Some(Default::default()),
        RenderGraphImageConstraint {
            samples: Some(RafxSampleCount::SampleCount1),
            format: Some(context.graph_config.color_format),
            ..Default::default()
        },
        Default::default(),
    );
    context.graph.set_image_name(blur_dst, "blur_dst");

    let sample_image =
        context
            .graph
            .sample_image(node, blur_src, Default::default(), Default::default());
    context.graph.set_image_name(blur_src, "blur_src");

    let bloom_blur_material_pass = bloom_blur_material_pass.clone();
    context.graph.set_renderpass_callback(node, move |args| {
        // Get the color image from before
        let sample_image = args.graph_context.image_view(sample_image);

        // Get the pipeline
        let pipeline = args
            .graph_context
            .resource_context()
            .graphics_pipeline_cache()
            .get_or_create_graphics_pipeline(
                PostProcessRenderPhase::render_phase_index(),
                &bloom_blur_material_pass,
                &args.render_target_meta,
                &EMPTY_VERTEX_LAYOUT,
            )?;

        let descriptor_set_layouts = &pipeline.get_raw().descriptor_set_layouts;

        // Set up a descriptor set pointing at the image so we can sample from it
        let mut descriptor_set_allocator = args
            .graph_context
            .resource_context()
            .create_descriptor_set_allocator();

        let horizontal = if blur_direction == BlurDirection::Horizontal {
            1
        } else {
            0
        };

        let bloom_blur_material_dyn_set = descriptor_set_allocator.create_descriptor_set(
            &descriptor_set_layouts[shaders::bloom_blur_frag::TEX_DESCRIPTOR_SET_INDEX],
            shaders::bloom_blur_frag::DescriptorSet0Args {
                tex: sample_image.as_ref().unwrap(),
                config: &shaders::bloom_blur_frag::ConfigUniform {
                    horizontal,
                    ..Default::default()
                },
            },
        )?;

        // Explicit flush since we're going to use the descriptors immediately
        descriptor_set_allocator.flush_changes()?;

        // Draw calls
        let command_buffer = &args.command_buffer;
        command_buffer.cmd_bind_pipeline(&*pipeline.get_raw().pipeline)?;
        bloom_blur_material_dyn_set.bind(command_buffer)?;
        command_buffer.cmd_draw(3, 0)?;

        Ok(())
    });

    blur_dst
}
