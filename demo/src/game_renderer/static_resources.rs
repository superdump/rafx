use distill::loader::handle::Handle;
use rafx::api::RafxResult;
use rafx::assets::distill_impl::AssetResource;
use rafx::assets::{AssetManager, ComputePipelineAsset};
use rafx::assets::{ImageAsset, MaterialAsset};

#[derive(Clone)]
pub struct GameRendererStaticResources {
    pub sprite_material: Handle<MaterialAsset>,
    pub bloom_extract_material: Handle<MaterialAsset>,
    pub bloom_blur_material: Handle<MaterialAsset>,
    pub bloom_combine_material: Handle<MaterialAsset>,
    pub imgui_material: Handle<MaterialAsset>,
    pub skybox_material: Handle<MaterialAsset>,
    pub skybox_texture: Handle<ImageAsset>,
    pub compute_test: Handle<ComputePipelineAsset>,
}

impl GameRendererStaticResources {
    pub fn new(
        asset_resource: &mut AssetResource,
        asset_manager: &mut AssetManager,
    ) -> RafxResult<Self> {
        //
        // Sprite resources
        //
        let sprite_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/sprite.material");

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
        // ImGui resources
        //
        let imgui_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/imgui.material");

        //
        // Skybox resources
        //
        let skybox_material =
            asset_resource.load_asset_path::<MaterialAsset, _>("materials/skybox.material");
        let skybox_texture =
            asset_resource.load_asset_path::<ImageAsset, _>("textures/skybox.basis");

        //
        // Compute pipeline
        //
        let compute_test = asset_resource
            .load_asset_path::<ComputePipelineAsset, _>("compute_pipelines/compute_test.compute");

        asset_manager.wait_for_asset_to_load(
            &sprite_material,
            asset_resource,
            "sprite_material",
        )?;

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

        asset_manager.wait_for_asset_to_load(&imgui_material, asset_resource, "imgui material")?;

        asset_manager.wait_for_asset_to_load(
            &skybox_material,
            asset_resource,
            "skybox material",
        )?;

        asset_manager.wait_for_asset_to_load(&skybox_texture, asset_resource, "skybox texture")?;

        asset_manager.wait_for_asset_to_load(&compute_test, asset_resource, "compute pipeline")?;

        Ok(GameRendererStaticResources {
            sprite_material,
            bloom_extract_material,
            bloom_blur_material,
            bloom_combine_material,
            imgui_material,
            skybox_material,
            skybox_texture,
            compute_test,
        })
    }
}
