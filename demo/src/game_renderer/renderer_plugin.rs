use crate::features::sprite::SpriteRenderNodeSet;
use rafx::api::extra::upload::RafxTransferUpload;
use rafx::api::RafxResult;
use rafx::assets::distill::daemon::AssetDaemon;
use rafx::assets::distill_impl::AssetResource;
use rafx::assets::AssetManager;
use rafx::base::resource_map::ResourceMap;
use rafx::framework::RenderResources;
use rafx::nodes::{ExtractJob, ExtractResources, RenderNodeReservations, RenderRegistryBuilder};

// graph builder?

pub trait RendererPlugin: Send {
    //
    fn configure_asset_daemon(
        &self,
        asset_daemon: AssetDaemon,
    ) -> AssetDaemon {
        asset_daemon
    }

    fn configure_render_registry(
        &self,
        render_registry: RenderRegistryBuilder,
    ) -> RenderRegistryBuilder {
        render_registry
    }

    // also add to render resources?
    fn configure_asset_manager(&self) {}

    fn initialize_static_resources(
        &mut self,
        _asset_manager: &mut AssetManager,
        _asset_resource: &mut AssetResource,
        _extract_resources: &ExtractResources,
        _render_resources: &mut ResourceMap,
        _upload: &mut RafxTransferUpload,
    ) -> RafxResult<()> {
        Ok(())
    }

    fn swapchain_created(
        &self,
        _extract_resources: &ExtractResources,
    ) -> RafxResult<()> {
        Ok(())
    }

    fn swapchain_destroyed(
        &self,
        _extract_resources: &ExtractResources,
    ) -> RafxResult<()> {
        Ok(())
    }

    fn add_render_node_reservations(
        &self,
        render_node_reservations: &mut RenderNodeReservations,
        extract_resources: &ExtractResources,
    ) {
        let mut sprite_render_nodes = extract_resources.fetch_mut::<SpriteRenderNodeSet>();
        sprite_render_nodes.update();
        render_node_reservations.add_reservation(&*sprite_render_nodes);
    }

    // build frame packet
    // generate views? visibility? or just calls to frame_packet_builder.add_view()?
    // extract jobs
    fn prepare_render(
        &self,
        _extract_resources: &ExtractResources,
    ) -> RafxResult<()> {
        Ok(())
    }

    fn add_extract_jobs(
        &self,
        _extract_resources: &ExtractResources,
        _render_resources: &RenderResources,
        _extract_jobs: &mut Vec<Box<dyn ExtractJob>>,
    ) {
    }
}
