//! A Storage wrapping a byte array in memory.

use super::{IStorage, Storage};
use crate::{sync_impl::RwLock, SwonchResult};
use alloc::vec::Vec;


/// A Storage wrapping a byte array in memory.
#[derive(Debug)]
pub enum VecStorage {
    ReadOnly(Vec<u8>),
    Mutable(RwLock<Vec<u8>>),
}

use VecStorage::*;

impl VecStorage {
    pub fn new(buf: Vec<u8>) -> Storage {
        Storage::new(Self::ReadOnly(buf))
    }

    pub fn new_mut(buf: Vec<u8>) -> Storage {
        Storage::new(Self::Mutable(RwLock::new(buf)))
    }

    pub fn map_inner<R>(&self, mut f: impl FnMut(&[u8]) -> R) -> R {
        match self {
            ReadOnly(v) => f(v.as_ref()),
            Mutable(v) => f(v.read().as_ref()),
        }
    }

    pub fn map_inner_mut<R>(&self, mut f: impl FnMut(&mut [u8]) -> R) -> Option<R> {
        match self {
            ReadOnly(_v) => None,
            Mutable(v) => Some(f(v.write().as_mut())),
        }
    }
}

impl IStorage for VecStorage {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<u64> {
        self.map_inner(|inner| {
            if let Some(available_buf) = inner.get(offset as usize..) {
                let avail_len = available_buf.len();
                let buf_len = buf.len();
                let read_len = core::cmp::min(avail_len, buf_len);
                let buf = &mut buf[..read_len];
                buf.copy_from_slice(&available_buf[..read_len]);
                return Ok(read_len as _);
            }
            Ok(0)
        })
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> SwonchResult<u64> {
        let ret = self.map_inner_mut(|buf| {
            let avail_size = buf.len().saturating_sub(offset as usize);
            let len_to_copy = core::cmp::min(data.len(), avail_size);
            if avail_size == 0 || len_to_copy == 0 {
                return Ok(0);
            }
            buf[offset as usize..][..len_to_copy].copy_from_slice(&data[..len_to_copy]);
            Ok(len_to_copy as _)
        });

        match ret {
            Some(res) => res,
            None => Err(crate::SwonchError::StorageIsReadOnly),
        }
    }

    fn length(&self) -> SwonchResult<u64> {
        Ok(self.map_inner(|inner| inner.len() as _))
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::RwLock;

    use super::{IStorage, SwonchResult, VecStorage};

    #[test]
    fn works_as_expected() -> SwonchResult<()> {
        let storage = VecStorage::ReadOnly(vec![1, 2, 3, 4]);

        // partial read from start
        let mut buf = [0; 3];
        storage.read_at(0, &mut buf)?;
        assert_eq!(buf, [1, 2, 3]);

        // full read from start
        let mut buf = [0; 4];
        storage.read_at(0, &mut buf)?;
        assert_eq!(buf, [1, 2, 3, 4]);

        // full read with larger array
        let mut buf = [0; 5];
        storage.read_at(0, &mut buf)?;
        assert_eq!(buf, [1, 2, 3, 4, 0]);

        // full read from middle
        let mut buf = [0; 3];
        storage.read_at(1, &mut buf)?;
        assert_eq!(buf, [2, 3, 4]);

        // read at the end
        let mut buf = [0; 3];
        let ret = storage.read_at(4, &mut buf)?;
        assert_eq!(ret, 0);
        assert_eq!(buf, [0, 0, 0]);

        Ok(())
    }

    #[test]
    fn writing_works() -> SwonchResult<()> {
        let storage = VecStorage::Mutable(RwLock::new(vec![0; 6]));

        assert_eq!(storage.write_at(0, &[1, 2])?, 2);
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 0, 0, 0, 0]));

        assert_eq!(storage.write_at(2, &[3, 4, 5])?, 3);
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 3, 4, 5, 0]));

        assert_eq!(storage.write_at(5, &[6, 7])?, 1);
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 3, 4, 5, 6]));

        assert_eq!(storage.write_at(8, &[0])?, 0);
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 3, 4, 5, 6]));

        assert_eq!(storage.write_at(0, &[0, 0, 0, 0, 0, 0, 0])?, 6);
        storage.map_inner(|buf| assert_eq!(buf, &[0, 0, 0, 0, 0, 0]));

        Ok(())
    }
}
