use rafx::assets::{MaterialAsset, AssetManager};
use rafx::distill::loader::handle::Handle;
use rafx::nodes::{RenderRegistryBuilder, ExtractResources, ExtractJob};
use crate::features::debug3d::Debug3dRenderFeature;
use crate::game_renderer::RendererPlugin;
use rafx::assets::distill_impl::AssetResource;
use rafx::framework::RenderResources;
use rafx::base::resource_map::ResourceMap;
use rafx::api::RafxResult;

pub struct Debug3DStaticResources {
    pub debug3d_material: Handle<MaterialAsset>,
}

#[derive(Default)]
pub struct Debug3DRendererPlugin {

}

impl RendererPlugin for Debug3DRendererPlugin {
    fn configure_render_registry(&self, render_registry: RenderRegistryBuilder) -> RenderRegistryBuilder {
        render_registry.register_feature::<Debug3dRenderFeature>()
    }

    fn initialize_static_resources(
        &mut self,
        asset_manager: &mut AssetManager,
        asset_resource: &mut AssetResource,
        render_resources: &mut ResourceMap,
    ) -> RafxResult<()> {
        let debug3d_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/debug.material");

        asset_manager.wait_for_asset_to_load(&debug3d_material, asset_resource, "debug.material").unwrap();

        render_resources.insert(Debug3DStaticResources {
            debug3d_material
        });

        Ok(())
    }

    fn add_extract_jobs(
        &self,
        _extract_resources: &ExtractResources,
        _render_resources: &RenderResources,
        extract_jobs: &mut Vec<Box<dyn ExtractJob>>
    ) {
        extract_jobs.push(super::create_debug3d_extract_job());
    }
}
