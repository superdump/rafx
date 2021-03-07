use crate::features::debug3d::Debug3dRenderFeature;
use crate::game_renderer::RendererPlugin;
use rafx::api::extra::upload::RafxTransferUpload;
use rafx::api::RafxResult;
use rafx::assets::distill_impl::AssetResource;
use rafx::assets::AssetManager;
use rafx::base::resource_map::ResourceMap;
use rafx::framework::RenderResources;
use rafx::nodes::{ExtractJob, ExtractResources, RenderRegistryBuilder, RenderNodeReservations};
use crate::features::mesh::MeshRenderNodeSet;
use crate::features::mesh::shadow_map_resource::ShadowMapResource;

#[derive(Default)]
pub struct MeshRendererPlugin {}

impl RendererPlugin for MeshRendererPlugin {
    fn configure_render_registry(
        &self,
        render_registry: RenderRegistryBuilder,
    ) -> RenderRegistryBuilder {
        render_registry.register_feature::<Debug3dRenderFeature>()
    }

    fn initialize_static_resources(
        &mut self,
        _asset_manager: &mut AssetManager,
        _asset_resource: &mut AssetResource,
        _extract_resources: &ExtractResources,
        render_resources: &mut ResourceMap,
        _upload: &mut RafxTransferUpload,
    ) -> RafxResult<()> {
        render_resources.insert(ShadowMapResource::default());

        Ok(())
    }

    fn add_render_node_reservations(
        &self,
        render_node_reservations: &mut RenderNodeReservations,
        extract_resources: &ExtractResources,
    ) {
        let mut mesh_render_nodes = extract_resources.fetch_mut::<MeshRenderNodeSet>();
        mesh_render_nodes.update();
        render_node_reservations.add_reservation(&*mesh_render_nodes);
    }

    fn add_extract_jobs(
        &self,
        _extract_resources: &ExtractResources,
        _render_resources: &RenderResources,
        extract_jobs: &mut Vec<Box<dyn ExtractJob>>,
    ) {
        extract_jobs.push(super::create_mesh_extract_job());
    }
}
