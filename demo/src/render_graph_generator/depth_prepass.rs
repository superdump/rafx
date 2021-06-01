use super::RenderGraphContext;
use crate::phases::DepthPrepassRenderPhase;
use rafx::api::RafxColorClearValue;
use rafx::api::RafxDepthStencilClearValue;
use rafx::api::RafxFormat;
use rafx::api::RafxResourceType;
use rafx::api::RafxSampleCount;
use rafx::graph::*;
use rafx::render_features::RenderJobCommandBufferContext;

pub(super) struct DepthPrepass {
    pub(super) node: RenderGraphNodeId,
    pub(super) depth: RenderGraphImageUsageId,
    pub(super) normal: RenderGraphImageUsageId,
}

pub(super) fn depth_prepass(context: &mut RenderGraphContext) -> DepthPrepass {
    let node = context
        .graph
        .add_node("DepthPrepass", RenderGraphQueue::DefaultGraphics);

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

    let normal = context.graph.create_color_attachment(
        node,
        0, // color attachment index
        Some(RafxColorClearValue([0.0, 0.0, 1.0, 1.0])),
        RenderGraphImageConstraint {
            samples: Some(RafxSampleCount::SampleCount1),
            format: Some(RafxFormat::R8G8B8A8_UNORM),
            resource_type: RafxResourceType::TEXTURE | RafxResourceType::RENDER_TARGET_COLOR,
            extents: Some(RenderGraphImageExtents::MatchSurface),
            layer_count: Some(1),
            mip_count: Some(1),
        },
        Default::default(),
    );
    context.graph.set_image_name(normal, "normal");

    context
        .graph
        .add_render_phase_dependency::<DepthPrepassRenderPhase>(node);

    let main_view = context.main_view.clone();

    context.graph.set_renderpass_callback(node, move |args| {
        profiling::scope!("Depth Prepass");
        let mut write_context =
            RenderJobCommandBufferContext::from_graph_visit_render_pass_args(&args);
        args.graph_context
            .prepared_render_data()
            .write_view_phase::<DepthPrepassRenderPhase>(&main_view, &mut write_context)
    });

    DepthPrepass {
        node,
        depth,
        normal,
    }
}
