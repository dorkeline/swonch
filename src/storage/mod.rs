use alloc::sync::Arc;

#[cfg(feature = "std")]
mod file;
#[cfg(feature = "std")]
pub use file::FileStorage;

use crate::{sync_impl::Mutex, SwonchResult};
use binrw::io::{Read, Seek, SeekFrom};

mod mapper;
mod memory;
mod substorage;

pub use self::{mapper::StorageMapper, memory::MemoryStorage, substorage::SubStorage};

pub trait Storage {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<usize>;

    fn length(&self) -> u64;

    fn split(self: &Arc<Self>, offset: u64, len: u64) -> Arc<SubStorage<Self>> {
        SubStorage::split_from(Arc::clone(self), offset, len)
    }

    fn map<M: StorageMapper<Self>>(self: &Arc<Self>, opts: M::Options) -> M::Output {
        M::map_from_storage(self, opts)
    }

    fn to_file_like(self: &Arc<Self>) -> StorageWrapper<Arc<Self>> {
        StorageWrapper {
            s: Arc::clone(self),
            offset: 0,
        }
    }
}

impl<S: ?Sized + Storage> Storage for Arc<S> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<usize> {
        (**self).read_at(offset, buf)
    }

    fn length(&self) -> u64 {
        (**self).length()
    }
}

impl<S: ?Sized + Storage> Storage for &S {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<usize> {
        (**self).read_at(offset, buf)
    }

    fn length(&self) -> u64 {
        (**self).length()
    }
}

impl<S: ?Sized + Storage> Storage for Mutex<S> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<usize> {
        (*self.lock()).read_at(offset, buf)
    }

    fn length(&self) -> u64 {
        (*self.lock()).length()
    }
}

pub trait WriteStorage: Storage {
    fn write_at(self: &Arc<Self>, offset: u64, data: &[u8]) -> SwonchResult<usize>;
}

/// wrap a Storage to get a type providing Read/Seek/Write implementations
pub struct StorageWrapper<S: Storage> {
    s: S,
    offset: u64,
}

impl<S: Storage> Read for StorageWrapper<S> {
    fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
        self.s
            .read_at(self.offset, buf)
            .map(|size| {
                self.offset += size as u64;
                size
            })
            .map_err(|e| binrw::io::Error::new(binrw::io::ErrorKind::Other, e))
    }
}

impl<S: Storage> Seek for StorageWrapper<S> {
    fn seek(&mut self, pos: SeekFrom) -> binrw::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.offset = offset,
            SeekFrom::Current(off) => {
                self.offset = (self.offset as i64 + off) as u64;
            }
            SeekFrom::End(off) => {
                self.offset = (self.s.length() as i64 - off) as u64;
            }
        };

        Ok(self.offset)
    }
}
