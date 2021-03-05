use crossbeam_channel::Sender;
use distill::loader::handle::{AssetHandle, GenericHandle};
use distill::loader::handle::Handle;
use distill::loader::storage::AssetLoadOp;
use distill::loader::{Loader, LoadHandle};
use rafx_api::RafxResult;
use crate::{AssetLookup, DynAssetLookup, AssetManager, GenericLoader, LoadQueues};
use std::sync::Arc;
use rafx_base::resource_map::ResourceMap;
use crate::distill_impl::{AssetResource, ResourceAssetLoader};
use fnv::FnvHashMap;
use type_uuid::TypeUuid;
use std::any::TypeId;
use std::marker::PhantomData;


pub trait AssetTypeFactory {
    fn create(asset_resource: &mut AssetResource) -> Box<dyn AssetType>;
}

pub trait AssetType {
    fn process_load_requests(
        &mut self,
        asset_manager: &AssetManager,
    );

    fn asset_lookup(
        &self
    ) -> &dyn DynAssetLookup;

    fn asset_type_id(&self) -> TypeId;
}












pub trait SimpleAssetTypeLoadHandler<AssetDataT, AssetT> {
    fn load(
        asset_manager: &AssetManager,
        font_asset: AssetDataT,
    ) -> RafxResult<AssetT>;
}

pub struct SimpleAssetTypeWithLoader<AssetDataT, AssetT, LoadHandlerT>
    where LoadHandlerT: SimpleAssetTypeLoadHandler<AssetDataT, AssetT>
{
    asset_lookup: AssetLookup<AssetT>,
    load_queues: LoadQueues<AssetDataT, AssetT>,
    phantom_data: PhantomData<LoadHandlerT>,
}

impl<AssetDataT, AssetT, LoadHandlerT> AssetTypeFactory for SimpleAssetTypeWithLoader<AssetDataT, AssetT, LoadHandlerT>
    where
        AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send + Clone,
        AssetT: TypeUuid + 'static + Send + Clone,
        LoadHandlerT: SimpleAssetTypeLoadHandler<AssetDataT, AssetT> + 'static
{
    fn create(asset_resource: &mut AssetResource) -> Box<dyn AssetType> {
        let load_queues = LoadQueues::<AssetDataT, AssetT>::default();

        asset_resource.add_storage_with_loader::<AssetDataT, AssetT, _>(Box::new(
            ResourceAssetLoader(load_queues.create_loader()),
        ));

        Box::new(Self {
            asset_lookup: AssetLookup::new(asset_resource.loader()),
            load_queues,
            phantom_data: Default::default()
        })
    }
}

impl<AssetDataT, AssetT, LoadHandlerT> AssetType for SimpleAssetTypeWithLoader<AssetDataT, AssetT, LoadHandlerT>
    where
        AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send + Clone,
        AssetT: TypeUuid + 'static + Send + Clone,
        LoadHandlerT: SimpleAssetTypeLoadHandler<AssetDataT, AssetT> + 'static
{
    fn process_load_requests(&mut self, asset_manager: &AssetManager) {
        for request in self.load_queues.take_load_requests() {
            log::trace!("Create asset type {} {:?}", std::any::type_name::<AssetT>(), request.load_handle);
            let loaded_asset = LoadHandlerT::load(asset_manager, request.asset);
            handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.asset_lookup,
                request.result_tx,
            );
        }

        handle_commit_requests(&mut self.load_queues, &mut self.asset_lookup);
        handle_free_requests(&mut self.load_queues, &mut self.asset_lookup);
    }

    fn asset_lookup(
        &self,
    ) -> &dyn DynAssetLookup {
        &self.asset_lookup
    }

    fn asset_type_id(&self) -> TypeId {
        TypeId::of::<AssetT>()
    }
}


fn handle_load_result<AssetT: Clone>(
    load_op: AssetLoadOp,
    loaded_asset: RafxResult<AssetT>,
    asset_lookup: &mut AssetLookup<AssetT>,
    result_tx: Sender<AssetT>,
) {
    match loaded_asset {
        Ok(loaded_asset) => {
            asset_lookup.set_uncommitted(load_op.load_handle(), loaded_asset.clone());
            result_tx.send(loaded_asset).unwrap();
            load_op.complete()
        }
        Err(err) => {
            load_op.error(err);
        }
    }
}

fn handle_commit_requests<AssetDataT, AssetT>(
    load_queues: &mut LoadQueues<AssetDataT, AssetT>,
    asset_lookup: &mut AssetLookup<AssetT>,
) {
    for request in load_queues.take_commit_requests() {
        log::trace!(
            "commit asset {:?} {}",
            request.load_handle,
            core::any::type_name::<AssetDataT>()
        );
        asset_lookup.commit(request.load_handle);
    }
}

fn handle_free_requests<AssetDataT, AssetT>(
    load_queues: &mut LoadQueues<AssetDataT, AssetT>,
    asset_lookup: &mut AssetLookup<AssetT>,
) {
    for request in load_queues.take_commit_requests() {
        asset_lookup.commit(request.load_handle);
    }
}