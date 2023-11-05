#[cfg(feature = "std")]
mod file;
use core::any::Any;

#[cfg(feature = "std")]
pub use file::FileStorage;

use crate::SwonchResult;

#[cfg(not(feature = "arc_storage"))]
use alloc::rc::Rc;

pub mod mapper;
mod memory;
pub mod stdio;
pub mod substorage;

pub use self::{
    mapper::FromStorage, memory::VecStorage, stdio::StorageStdioWrapper, substorage::SubStorage,
};

pub trait IStorage: Any + core::fmt::Debug + 'static {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<u64>;

    fn write_at(&self, _offset: u64, _data: &[u8]) -> SwonchResult<u64> {
        // by default we are not writeable
        Err(crate::SwonchError::StorageIsReadOnly)
    }

    fn split(self, offset: u64, len: u64) -> SwonchResult<Storage>
    where
        Self: Sized,
    {
        Ok(SubStorage::split_from(Storage::new(self), offset, len)?)
    }

    fn into_stdio(self) -> StorageStdioWrapper
    where
        Self: Sized,
    {
        Storage::new(self).into_stdio()
    }

    fn length(&self) -> SwonchResult<u64>;

    fn into_storage(self) -> Storage
    where
        Self: Sized,
    {
        Storage::new(self)
    }
}

#[derive(Debug)]
pub struct Storage {
    #[cfg(not(feature = "arc_storage"))]
    inner: alloc::rc::Rc<dyn IStorage>,

    #[cfg(feature = "arc_storage")]
    inner: alloc::sync::Arc<dyn IStorage>,
}

pub trait IntoRcStorage {
    fn into_rc_storage(self) -> Rc<dyn IStorage>;
}

impl IntoRcStorage for Rc<Storage> {
    fn into_rc_storage(self) -> Rc<dyn IStorage> {
        self
    }
}

impl<T: IStorage + 'static> IntoRcStorage for T {
    fn into_rc_storage(self) -> Rc<dyn IStorage> {
        Rc::new(self)
    }
}

impl Storage {
    pub fn new(storage: impl IntoRcStorage) -> Self {
        Self {
            #[cfg(not(feature = "arc_storage"))]
            inner: storage.into_rc_storage(),

            #[cfg(feature = "arc_storage")]
            inner: alloc::sync::Arc::new(storage),
        }
    }

    pub fn map_to_storage<M: FromStorage>(self, args: M::Args) -> SwonchResult<M> {
        M::from_storage(self, args)
    }

    pub fn split(self, offset: u64, len: u64) -> SwonchResult<Storage> {
        Ok(SubStorage::split_from(self, offset, len)?)
    }

    pub fn into_stdio(self) -> StorageStdioWrapper {
        StorageStdioWrapper::new(self)
    }
}

impl IStorage for Storage {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<u64> {
        self.inner.read_at(offset, buf)
    }

    fn split(self, offset: u64, len: u64) -> SwonchResult<Storage> {
        Self::split(self, offset, len)
    }

    fn into_stdio(self) -> StorageStdioWrapper
    where
        Self: Sized,
    {
        Self::into_stdio(self)
    }

    fn length(&self) -> SwonchResult<u64> {
        self.inner.length()
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
