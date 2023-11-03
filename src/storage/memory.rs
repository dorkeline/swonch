//! A Storage wrapping a byte array in memory.

use super::{Storage, WriteStorage};
use alloc::sync::Arc;
use core::fmt;
use parking_lot::Mutex;

/// A Storage wrapping a byte array in memory.
pub struct MemoryStorage<B>(Mutex<B>);

impl<B: AsRef<[u8]>> MemoryStorage<B> {
    /// Creates a new MemoryStorage from a byte array.
    ///
    /// # Examples
    /// ```
    /// use swonch::storage::{Storage, MemoryStorage};
    ///
    /// let array1 = [1, 2, 3];
    /// let array2 = vec![1, 2, 3];
    ///
    /// let storage1 = MemoryStorage::new(array1);
    /// let storage2 = MemoryStorage::new(array2);
    ///
    /// let mut buf1 = [0; 3];
    /// storage1.read_at(0, &mut buf1).unwrap();
    /// let mut buf2 = [0; 3];
    /// storage2.read_at(0, &mut buf2).unwrap();
    ///
    /// assert_eq!(buf1, buf2);
    /// ```
    pub fn new(backing: B) -> Arc<Self> {
        Arc::new(Self(Mutex::new(backing)))
    }

    // FIXME: this is only needed because the inner is a Mutex, can we get rid of this (and/or the Mutex)
    // while still having write support somehow? maybe bytes::{Bytes, BytesMut} and specialised impl's?
    pub fn map_inner(&self, f: impl FnOnce(&[u8])) {
        let lock = self.0.lock();
        f(lock.as_ref());
    }
}

impl MemoryStorage<Vec<u8>> {
    pub fn with_capacity(cap: usize) -> Arc<Self> {
        Self::new(vec![0; cap])
    }
}

impl<B: AsRef<[u8]>> Storage for MemoryStorage<B> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ()> {
        let lock = self.0.lock();
        let inner = lock.as_ref();
        if let Some(available_buf) = inner.get(offset as usize..) {
            let avail_len = available_buf.len();
            let buf_len = buf.len();
            let read_len = core::cmp::min(avail_len, buf_len);
            let buf = &mut buf[..read_len];
            buf.copy_from_slice(&available_buf[..read_len]);
            return Ok(read_len);
        }
        Ok(0)
    }

    fn length(&self) -> u64 {
        self.0.lock().as_ref().len() as _
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> WriteStorage for MemoryStorage<B> {
    fn write_at(self: &Arc<Self>, offset: u64, data: &[u8]) -> Result<usize, ()> {
        let mut lock = self.0.lock();
        let buf = lock.as_mut();
        let avail_size = buf.len().checked_sub(offset as usize).unwrap_or(0);
        let len_to_copy = core::cmp::min(data.len(), avail_size);
        if avail_size == 0 || len_to_copy == 0 {
            return Ok(0)
        }
        buf[offset as usize..][..len_to_copy].copy_from_slice(&data[..len_to_copy]);
        Ok(len_to_copy)
    }
}

impl<B: Clone + AsRef<[u8]>> fmt::Debug for MemoryStorage<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MemoryBackend")
            .field(&self.0.lock().as_ref())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{MemoryStorage, Storage, WriteStorage};

    #[test]
    fn works_as_expected() {
        let v = [1, 2, 3, 4].to_vec();
        let storage = MemoryStorage::new(v);

        // partial read from start
        let mut buf = [0; 3];
        storage.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3]);

        // full read from start
        let mut buf = [0; 4];
        storage.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3, 4]);

        // full read with larger array
        let mut buf = [0; 5];
        storage.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3, 4, 0]);

        // full read from middle
        let mut buf = [0; 3];
        storage.read_at(1, &mut buf).unwrap();
        assert_eq!(buf, [2, 3, 4]);

        // read at the end
        let mut buf = [0; 3];
        let ret = storage.read_at(4, &mut buf).unwrap();
        assert_eq!(ret, 0);
        assert_eq!(buf, [0, 0, 0]);
    }

    #[test]
    fn writing_works() {
        let storage = MemoryStorage::with_capacity(6);

        assert_eq!(storage.write_at(0, &[1, 2]), Ok(2));
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 0, 0, 0, 0]));

        assert_eq!(storage.write_at(2, &[3, 4, 5]), Ok(3));
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 3, 4, 5, 0]));

        assert_eq!(storage.write_at(5, &[6, 7]), Ok(1));
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 3, 4, 5, 6]));

        assert_eq!(storage.write_at(8, &[0]), Ok(0));
        storage.map_inner(|buf| assert_eq!(buf, &[1, 2, 3, 4, 5, 6]));

        assert_eq!(storage.write_at(0, &[0, 0, 0, 0, 0, 0, 0]), Ok(6));
        storage.map_inner(|buf| assert_eq!(buf, &[0, 0, 0, 0, 0, 0]));
    }
}
