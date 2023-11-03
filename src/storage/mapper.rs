use alloc::sync::Arc;

use super::Storage;

pub trait StorageMapper<S: Storage + ?Sized> {
    type Options;
    type Output;

    fn map_from_storage(s: &Arc<S>, opts: Self::Options) -> Self::Output;
}
