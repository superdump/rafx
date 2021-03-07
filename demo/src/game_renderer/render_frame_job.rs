use crate::game_renderer::GameRenderer;
use rafx::api::{RafxCommandBuffer, RafxDeviceContext, RafxQueue};
use rafx::api::{RafxPresentableFrame, RafxResult};
use rafx::framework::{DynCommandBuffer, RenderResources, ResourceContext};
use rafx::graph::RenderGraphExecutor;
use rafx::nodes::{
    FramePacket, PrepareJobSet, RenderJobPrepareContext, RenderRegistry, RenderView,
};
use crate::features::mesh::shadow_map_resource::ShadowMapResource;

pub struct RenderFrameJobResult;

pub struct RenderFrameJob {
    pub game_renderer: GameRenderer,
    pub prepare_job_set: PrepareJobSet,
    pub render_graph: RenderGraphExecutor,
    pub resource_context: ResourceContext,
    pub frame_packet: FramePacket,
    pub main_view: RenderView,
    pub render_registry: RenderRegistry,
    pub device_context: RafxDeviceContext,
    pub graphics_queue: RafxQueue,
}

impl RenderFrameJob {
    pub fn render_async(
        self,
        presentable_frame: RafxPresentableFrame,
        render_resources: &RenderResources,
    ) -> RenderFrameJobResult {
        let t0 = std::time::Instant::now();
        let result = Self::do_render_async(
            self.prepare_job_set,
            self.render_graph,
            self.resource_context,
            self.frame_packet,
            self.main_view,
            self.render_registry,
            render_resources,
            self.graphics_queue,
        );

        let t1 = std::time::Instant::now();
        log::trace!(
            "[render thread] render took {} ms",
            (t1 - t0).as_secs_f32() * 1000.0
        );

        match result {
            Ok(command_buffers) => {
                // ignore the error, we will receive it when we try to acquire the next image
                let graphics_queue = self.game_renderer.graphics_queue();

                let refs: Vec<&RafxCommandBuffer> = command_buffers.iter().map(|x| &**x).collect();
                let _ = presentable_frame.present(graphics_queue, &refs);
            }
            Err(err) => {
                log::error!("Render thread failed with error {:?}", err);
                // Pass error on to the next swapchain image acquire call
                let graphics_queue = self.game_renderer.graphics_queue();
                presentable_frame.present_with_error(graphics_queue, err);
            }
        }

        let t2 = std::time::Instant::now();
        log::trace!(
            "[render thread] present took {} ms",
            (t2 - t1).as_secs_f32() * 1000.0
        );

        RenderFrameJobResult {}
    }

    #[allow(clippy::too_many_arguments)]
    fn do_render_async(
        prepare_job_set: PrepareJobSet,
        render_graph: RenderGraphExecutor,
        resource_context: ResourceContext,
        frame_packet: FramePacket,
        main_view: RenderView,
        render_registry: RenderRegistry,
        render_resources: &RenderResources,
        graphics_queue: RafxQueue,
    ) -> RafxResult<Vec<DynCommandBuffer>> {
        let t0 = std::time::Instant::now();

        //
        // Prepare Jobs - everything beyond this point could be done in parallel with the main thread
        //
        let prepared_render_data = {
            profiling::scope!("Renderer Prepare");

            let mut prepare_views = Vec::default();
            prepare_views.push(main_view);

            let shadow_map_resource = render_resources.fetch::<ShadowMapResource>();
            shadow_map_resource.append_render_views(&mut prepare_views);

            let prepare_context =
                RenderJobPrepareContext::new(resource_context.clone(), &render_resources);

            prepare_job_set.prepare(
                &prepare_context,
                &frame_packet,
                &prepare_views,
                &render_registry,
            )
        };
        let t1 = std::time::Instant::now();
        log::trace!(
            "[render thread] render prepare took {} ms",
            (t1 - t0).as_secs_f32() * 1000.0
        );

        let command_buffers = {
            profiling::scope!("Renderer Execute Graph");
            render_graph.execute_graph(prepared_render_data, &graphics_queue)?
        };
        let t2 = std::time::Instant::now();
        log::trace!(
            "[render thread] execute graph took {} ms",
            (t2 - t1).as_secs_f32() * 1000.0
        );

        Ok(command_buffers)
    }
}
