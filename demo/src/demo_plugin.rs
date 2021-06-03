use crate::phases::{
    DepthPrepassRenderPhase, OpaqueRenderPhase, PostProcessRenderPhase, ShadowMapRenderPhase,
    TransparentRenderPhase, UiRenderPhase, WireframeRenderPhase,
};
use rafx::api::extra::upload::RafxTransferUpload;
use rafx::api::RafxResult;
use rafx::assets::distill_impl::AssetResource;
use rafx::assets::{AssetManager, ComputePipelineAsset, MaterialAsset};
use rafx::base::resource_map::ResourceMap;
use rafx::distill::loader::handle::Handle;
use rafx::render_features::{ExtractResources, RenderRegistryBuilder};
use rafx::renderer::RendererAssetPlugin;

// A plugin that add demo-specific configuration

pub struct DemoStaticResources {
    pub bloom_extract_material: Handle<MaterialAsset>,
    pub bloom_blur_material: Handle<MaterialAsset>,
    pub bloom_combine_material: Handle<MaterialAsset>,
    pub compute_test: Handle<ComputePipelineAsset>,
    pub ssao_material: Handle<MaterialAsset>,
}

pub struct DemoRendererPlugin;

impl RendererAssetPlugin for DemoRendererPlugin {
    fn configure_render_registry(
        &self,
        render_registry_builder: RenderRegistryBuilder,
    ) -> RenderRegistryBuilder {
        render_registry_builder
            .register_render_phase::<DepthPrepassRenderPhase>("DepthPrepass")
            .register_render_phase::<ShadowMapRenderPhase>("ShadowMap")
            .register_render_phase::<OpaqueRenderPhase>("Opaque")
            .register_render_phase::<TransparentRenderPhase>("Transparent")
            .register_render_phase::<WireframeRenderPhase>("Wireframe")
            .register_render_phase::<PostProcessRenderPhase>("PostProcess")
            .register_render_phase::<UiRenderPhase>("Ui")
    }

    fn initialize_static_resources(
        &self,
        asset_manager: &mut AssetManager,
        asset_resource: &mut AssetResource,
        _extract_resources: &ExtractResources,
        render_resources: &mut ResourceMap,
        _upload: &mut RafxTransferUpload,
    ) -> RafxResult<()> {
        //
        // Bloom extract resources
        //
        // let bloom_extract_material = asset_resource
        //     .load_asset_path::<MaterialAsset, _>("pipelines/bloom_extract.material");
        let bloom_extract_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/bloom_extract.material");
        //.load_asset::<MaterialAsset>(asset_uuid!("4c5509e3-4a9f-45c2-a6dc-862a925d2341"));

        //
        // Bloom blur resources
        //
        let bloom_blur_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/bloom_blur.material");

        //
        // Bloom combine resources
        //
        let bloom_combine_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/bloom_combine.material");

        //
        // Compute pipeline
        //
        let compute_test = asset_resource
            .load_asset_path::<ComputePipelineAsset, _>("compute_pipelines/compute_test.compute");

        //
        // SSAO resources
        //
        let ssao_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/ssao.material");

        asset_manager.wait_for_asset_to_load(
            &bloom_extract_material,
            asset_resource,
            "bloom extract material",
        )?;

        asset_manager.wait_for_asset_to_load(
            &bloom_blur_material,
            asset_resource,
            "bloom blur material",
        )?;

        asset_manager.wait_for_asset_to_load(
            &bloom_combine_material,
            asset_resource,
            "bloom combine material",
        )?;

        asset_manager.wait_for_asset_to_load(&compute_test, asset_resource, "compute pipeline")?;

        asset_manager.wait_for_asset_to_load(&ssao_material, asset_resource, "ssao material")?;

        render_resources.insert(DemoStaticResources {
            bloom_extract_material,
            bloom_blur_material,
            bloom_combine_material,
            compute_test,
            ssao_material,
        });

        Ok(())
    }
}
