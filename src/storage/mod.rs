

#[cfg(feature = "std")]
mod file;
#[cfg(feature = "std")]
pub use file::FileStorage;

use crate::SwonchResult;
use binrw::io::{Read, Seek, SeekFrom};

#[cfg(not(feature = "arc_storage"))]
use alloc::rc::Rc;

pub mod mapper;
mod memory;
pub mod substorage;

pub use self::{mapper::FromStorage, memory::VecStorage, substorage::SubStorage};

pub trait IStorage: core::fmt::Debug + 'static {
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
        StorageStdioWrapper { s: self, offset: 0 }
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

/// wrap a Storage to get a type providing Read/Seek/Write implementations
pub struct StorageStdioWrapper {
    s: Storage,
    offset: u64,
}

impl Read for StorageStdioWrapper {
    fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
        self.s
            .read_at(self.offset, buf)
            .map(|size| {
                self.offset += size as u64;
                size as _
            })
            .map_err(crate::utils::other_io_error)
    }
}

impl Seek for StorageStdioWrapper {
    fn seek(&mut self, pos: SeekFrom) -> binrw::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.offset = offset,
            SeekFrom::Current(off) => {
                self.offset = (self.offset as i64 + off) as u64;
            }
            SeekFrom::End(off) => {
                self.offset = (self.s.length()? as i64 - off) as u64;
            }
        };

        Ok(self.offset)
    }
}
