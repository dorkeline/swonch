use alloc::sync::Arc;

#[cfg(feature = "std")]
mod file;
#[cfg(feature = "std")]
pub use file::FileStorage;

use binrw::io::{Read, Seek, SeekFrom};

mod mapper;
mod memory;
mod substorage;

pub use self::{mapper::StorageMapper, memory::MemoryStorage, substorage::SubStorage};

pub trait Storage {
    fn read_at(self: &Arc<Self>, offset: u64, buf: &mut [u8]) -> Result<usize, ()>;

    fn length(&self) -> u64;

    fn split(self: &Arc<Self>, offset: u64, len: u64) -> Arc<SubStorage<Self>> {
        SubStorage::split_from(Arc::clone(self), offset, len)
    }

    fn map<M: StorageMapper<Self>>(self: &Arc<Self>, opts: M::Options) -> M::Output {
        M::map_from_storage(self, opts)
    }

    fn to_file_like(self: &Arc<Self>) -> StorageWrapper<Self> {
        StorageWrapper {
            s: Arc::clone(self),
            offset: 0,
        }
    }
}

impl<S: ?Sized + Storage> Storage for Arc<S> {
    fn read_at(self: &Arc<Self>, offset: u64, buf: &mut [u8]) -> Result<usize, ()> {
        (**self).read_at(offset, buf)
    }

    fn length(&self) -> u64 {
        (**self).length()
    }
}

pub trait WriteStorage: Storage {
    fn write_at(self: &Arc<Self>, offset: u64, data: &[u8]) -> Result<usize, ()>;
}

/// wrap a Storage to get a type providing Read/Seek/Write implementations
pub struct StorageWrapper<S: ?Sized + Storage> {
    s: Arc<S>,
    offset: u64,
}

impl<S: ?Sized + Storage> Read for StorageWrapper<S> {
    fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
        self.s
            .read_at(self.offset, buf)
            .map(|size| {
                self.offset += size as u64;
                size
            })
            .map_err(|_e| todo!())
    }
}

impl<S: ?Sized + Storage> Seek for StorageWrapper<S> {
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
