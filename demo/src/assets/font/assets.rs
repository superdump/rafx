use serde::{Deserialize, Serialize};
use type_uuid::*;
use std::sync::Arc;

#[derive(TypeUuid, Serialize, Deserialize, Clone)]
#[uuid = "197bfd7a-3df9-4440-86f0-8e10756c714e"]
pub struct FontAssetData {
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

pub struct FontAssetInner {
    pub data: Vec<u8>
}

#[derive(TypeUuid, Clone)]
#[uuid = "398689ef-4bf1-42ad-8fc4-5080c1b8293a"]
pub struct FontAsset {
    pub inner: Arc<FontAssetInner>
}
