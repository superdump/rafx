use rafx::api::RafxResult;
use rafx::assets::distill_impl::AssetResource;
use rafx::assets::AssetManager;
use rafx::base::resource_map::ResourceMap;
use rafx::framework::RenderResources;
use rafx::nodes::{ExtractJob, ExtractResources, RenderRegistryBuilder};

// graph builder?

pub trait RendererPlugin: Send {
    //
    fn configure_asset_daemon(&self) {}

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
        _render_resources: &mut ResourceMap,
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
