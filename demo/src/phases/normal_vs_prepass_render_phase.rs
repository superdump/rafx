use rafx::render_features::RenderPhase;
use rafx::render_features::{RenderFeatureSubmitNode, RenderPhaseIndex};

rafx::declare_render_phase!(
    NormalPrepassRenderPhase,
    NORMAL_PREPASS_RENDER_PHASE_INDEX,
    normal_prepass_render_phase_sort_submit_nodes
);

#[profiling::function]
fn normal_prepass_render_phase_sort_submit_nodes(submit_nodes: &mut Vec<RenderFeatureSubmitNode>) {
    // Sort by distance from camera front to back
    log::trace!(
        "Sort phase {}",
        NormalPrepassRenderPhase::render_phase_debug_name()
    );
    submit_nodes.sort_unstable_by(|a, b| a.distance().partial_cmp(&b.distance()).unwrap());
}
