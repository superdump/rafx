use crate::assets::font::{FontAsset, FontAssetData, FontAssetInner};
use crate::assets::gltf::MeshAssetData;
use crate::game_asset_lookup::{
    MeshAsset, MeshAssetInner, MeshAssetPart,
};
use crate::phases::{OpaqueRenderPhase, ShadowMapRenderPhase};
use crossbeam_channel::Sender;
use distill::loader::handle::{AssetHandle, GenericHandle};
use distill::loader::handle::Handle;
use distill::loader::storage::AssetLoadOp;
use distill::loader::{Loader, LoadHandle};
use fontdue::FontSettings;
use rafx::api::RafxResult;
use rafx::assets::{AssetLookup, DynAssetLookup, AssetManager, GenericLoader, LoadQueues, SimpleAssetTypeLoadHandler, SimpleAssetTypeWithLoader, AssetType, AssetTypeFactory};
use std::sync::Arc;
use rafx::base::resource_map::ResourceMap;
use rafx::assets::distill_impl::{AssetResource, ResourceAssetLoader};
use fnv::FnvHashMap;
use type_uuid::TypeUuid;
use std::any::TypeId;
use std::marker::PhantomData;

// pub trait AssetTypeFactory {
//     fn create(asset_resource: &mut AssetResource) -> Box<dyn AssetType>;
// }
//
// pub trait AssetType {
//     fn process_load_requests(
//         &mut self,
//         asset_manager: &AssetManager,
//     );
//
//     fn asset_lookup(
//         &self
//     ) -> &dyn DynAssetLookup;
//
//     fn asset_type_id(&self) -> TypeId;
// }
//
//
//
//
//
//
//
//
//
//
//
//
// pub trait SimpleAssetTypeLoadHandler<AssetDataT, AssetT> {
//     fn load(
//         asset_manager: &AssetManager,
//         font_asset: AssetDataT,
//     ) -> RafxResult<AssetT>;
// }
//
// pub struct SimpleAssetTypeWithLoader<AssetDataT, AssetT, LoadHandlerT>
//     where LoadHandlerT: SimpleAssetTypeLoadHandler<AssetDataT, AssetT>
// {
//     asset_lookup: AssetLookup<AssetT>,
//     load_queues: LoadQueues<AssetDataT, AssetT>,
//     phantom_data: PhantomData<LoadHandlerT>,
// }
//
// impl<AssetDataT, AssetT, LoadHandlerT> AssetTypeFactory for SimpleAssetTypeWithLoader<AssetDataT, AssetT, LoadHandlerT>
//     where
//         AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send + Clone,
//         AssetT: TypeUuid + 'static + Send + Clone,
//         LoadHandlerT: SimpleAssetTypeLoadHandler<AssetDataT, AssetT> + 'static
// {
//     fn create(asset_resource: &mut AssetResource) -> Box<dyn AssetType> {
//         let load_queues = LoadQueues::<AssetDataT, AssetT>::default();
//
//         asset_resource.add_storage_with_loader::<AssetDataT, AssetT, _>(Box::new(
//             ResourceAssetLoader(load_queues.create_loader()),
//         ));
//
//         Box::new(Self {
//             asset_lookup: AssetLookup::new(asset_resource.loader()),
//             load_queues,
//             phantom_data: Default::default()
//         })
//     }
// }
//
// impl<AssetDataT, AssetT, LoadHandlerT> AssetType for SimpleAssetTypeWithLoader<AssetDataT, AssetT, LoadHandlerT>
//     where
//         AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send + Clone,
//         AssetT: TypeUuid + 'static + Send + Clone,
//         LoadHandlerT: SimpleAssetTypeLoadHandler<AssetDataT, AssetT> + 'static
// {
//     fn process_load_requests(&mut self, asset_manager: &AssetManager) {
//         for request in self.load_queues.take_load_requests() {
//             log::trace!("Create asset type {} {:?}", std::any::type_name::<AssetT>(), request.load_handle);
//             let loaded_asset = LoadHandlerT::load(asset_manager, request.asset);
//             GameAssetManager::handle_load_result(
//                 request.load_op,
//                 loaded_asset,
//                 &mut self.asset_lookup,
//                 request.result_tx,
//             );
//         }
//
//         GameAssetManager::handle_commit_requests(&mut self.load_queues, &mut self.asset_lookup);
//         GameAssetManager::handle_free_requests(&mut self.load_queues, &mut self.asset_lookup);
//     }
//
//     fn asset_lookup(
//         &self,
//     ) -> &dyn DynAssetLookup {
//         &self.asset_lookup
//     }
//
//     fn asset_type_id(&self) -> TypeId {
//         TypeId::of::<AssetT>()
//     }
// }
//
//






pub struct FontLoadHandler;

impl SimpleAssetTypeLoadHandler<FontAssetData, FontAsset> for FontLoadHandler {
    #[profiling::function]
    fn load(
        _asset_manager: &AssetManager,
        font_asset: FontAssetData,
    ) -> RafxResult<FontAsset> {
        let settings = FontSettings::default();
        let font = fontdue::Font::from_bytes(font_asset.data.as_slice(), settings)
            .map_err(|x| x.to_string())?;

        let inner = FontAssetInner {
            font,
            data_hash: font_asset.data_hash,
            scale: font_asset.scale,
        };

        Ok(FontAsset {
            inner: Arc::new(inner),
        })
    }
}






pub struct MeshLoadHandler;

impl SimpleAssetTypeLoadHandler<MeshAssetData, MeshAsset> for MeshLoadHandler {
    #[profiling::function]
    fn load(
        asset_manager: &AssetManager,
        mesh_asset: MeshAssetData,
    ) -> RafxResult<MeshAsset> {
        let vertex_buffer = asset_manager
            .loaded_assets()
            .buffers
            .get_latest(mesh_asset.vertex_buffer.load_handle())
            .unwrap()
            .buffer
            .clone();
        let index_buffer = asset_manager
            .loaded_assets()
            .buffers
            .get_latest(mesh_asset.index_buffer.load_handle())
            .unwrap()
            .buffer
            .clone();

        let mesh_parts: Vec<_> = mesh_asset
            .mesh_parts
            .iter()
            .map(|mesh_part| {
                let material_instance = asset_manager
                    .loaded_assets()
                    .material_instances
                    .get_committed(mesh_part.material_instance.load_handle())
                    .unwrap();

                let opaque_pass_index = material_instance
                    .material
                    .find_pass_by_phase::<OpaqueRenderPhase>();
                if opaque_pass_index.is_none() {
                    log::error!(
                        "A mesh part with material {:?} has no opaque phase",
                        material_instance.material_handle
                    );
                    return None;
                }
                let opaque_pass_index = opaque_pass_index.unwrap();

                //NOTE: For now require this, but we might want to disable shadow casting, in which
                // case no material is necessary
                let shadow_map_pass_index = material_instance
                    .material
                    .find_pass_by_phase::<ShadowMapRenderPhase>();
                if shadow_map_pass_index.is_none() {
                    log::error!(
                        "A mesh part with material {:?} has no shadow map phase",
                        material_instance.material_handle
                    );
                    return None;
                }

                const PER_MATERIAL_DESCRIPTOR_SET_LAYOUT_INDEX: usize = 1;

                Some(MeshAssetPart {
                    opaque_pass: material_instance.material.passes[opaque_pass_index].clone(),
                    opaque_material_descriptor_set: material_instance.material_descriptor_sets
                        [opaque_pass_index][PER_MATERIAL_DESCRIPTOR_SET_LAYOUT_INDEX]
                        .as_ref()
                        .unwrap()
                        .clone(),
                    shadow_map_pass: shadow_map_pass_index
                        .map(|pass_index| material_instance.material.passes[pass_index].clone()),
                    vertex_buffer_offset_in_bytes: mesh_part.vertex_buffer_offset_in_bytes,
                    vertex_buffer_size_in_bytes: mesh_part.vertex_buffer_size_in_bytes,
                    index_buffer_offset_in_bytes: mesh_part.index_buffer_offset_in_bytes,
                    index_buffer_size_in_bytes: mesh_part.index_buffer_size_in_bytes,
                })
            })
            .collect();

        let inner = MeshAssetInner {
            vertex_buffer,
            index_buffer,
            asset_data: mesh_asset,
            mesh_parts,
        };

        Ok(MeshAsset {
            inner: Arc::new(inner),
        })
    }
}

pub type FontAssetType = SimpleAssetTypeWithLoader<FontAssetData, FontAsset, FontLoadHandler>;
pub type MeshAssetType = SimpleAssetTypeWithLoader<MeshAssetData, MeshAsset, MeshLoadHandler>;

#[derive(Debug)]
pub struct GameAssetManagerMetrics {
    //pub game_loaded_asset_metrics: GameLoadedAssetMetrics,
}

// #[derive(Default)]
// pub struct GameLoadQueueSet {
//     pub meshes: LoadQueues<MeshAssetData, MeshAsset>,
//     pub fonts: LoadQueues<FontAssetData, FontAsset>,
// }

pub struct GameAssetManager {
    asset_types: FnvHashMap<TypeId, Box<dyn AssetType>>,
}

impl GameAssetManager {
    pub fn new() -> Self {
        GameAssetManager {
            asset_types: Default::default(),
        }
    }

    pub fn add_asset_type<AssetTypeFactoryT: AssetTypeFactory>(&mut self, asset_resource: &mut AssetResource) {
        let asset_type = AssetTypeFactoryT::create(asset_resource);
        let old = self.asset_types.insert(asset_type.asset_type_id(), asset_type);
        assert!(old.is_none());
    }

    pub fn asset<AssetT: 'static>(
        &self,
        handle: &Handle<AssetT>
    ) -> Option<&AssetT> {
        let asset_type = self.asset_types.get(&TypeId::of::<AssetT>())?;
        asset_type.asset_lookup().downcast_ref::<AssetLookup<AssetT>>().unwrap().get_committed(handle.load_handle())
    }

    // Call whenever you want to handle assets loading/unloading
    #[profiling::function]
    pub fn update_asset_loaders(
        &mut self,
        asset_manager: &AssetManager,
    ) -> RafxResult<()> {
        // self.process_mesh_load_requests(asset_manager);
        // self.process_font_load_requests(asset_manager);

        for (ty, asset_type) in &mut self.asset_types {
            asset_type.process_load_requests(asset_manager);
        }

        Ok(())
    }

    // pub fn metrics(&self) -> GameAssetManagerMetrics {
    //     let game_loaded_asset_metrics = self.loaded_assets.metrics();
    //
    //     GameAssetManagerMetrics {
    //         game_loaded_asset_metrics,
    //     }
    // }
    //
    // fn handle_load_result<AssetT: Clone>(
    //     load_op: AssetLoadOp,
    //     loaded_asset: RafxResult<AssetT>,
    //     asset_lookup: &mut AssetLookup<AssetT>,
    //     result_tx: Sender<AssetT>,
    // ) {
    //     match loaded_asset {
    //         Ok(loaded_asset) => {
    //             asset_lookup.set_uncommitted(load_op.load_handle(), loaded_asset.clone());
    //             result_tx.send(loaded_asset).unwrap();
    //             load_op.complete()
    //         }
    //         Err(err) => {
    //             load_op.error(err);
    //         }
    //     }
    // }
    //
    // fn handle_commit_requests<AssetDataT, AssetT>(
    //     load_queues: &mut LoadQueues<AssetDataT, AssetT>,
    //     asset_lookup: &mut AssetLookup<AssetT>,
    // ) {
    //     for request in load_queues.take_commit_requests() {
    //         log::trace!(
    //             "commit asset {:?} {}",
    //             request.load_handle,
    //             core::any::type_name::<AssetDataT>()
    //         );
    //         asset_lookup.commit(request.load_handle);
    //     }
    // }
    //
    // fn handle_free_requests<AssetDataT, AssetT>(
    //     load_queues: &mut LoadQueues<AssetDataT, AssetT>,
    //     asset_lookup: &mut AssetLookup<AssetT>,
    // ) {
    //     for request in load_queues.take_commit_requests() {
    //         asset_lookup.commit(request.load_handle);
    //     }
    // }
}

impl Drop for GameAssetManager {
    fn drop(&mut self) {
        log::info!("Cleaning up game resource manager");
        //log::trace!("Game Resource Manager Metrics:\n{:#?}", self.metrics());

        // Wipe out any loaded assets. This will potentially drop ref counts on resources
        //self.loaded_assets.destroy();
        self.asset_types.clear();

        log::info!("Dropping game resource manager");
        //log::trace!("Resource Game Manager Metrics:\n{:#?}", self.metrics());
    }
}
