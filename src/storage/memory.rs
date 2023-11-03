//! A Storage wrapping a byte array in memory.

use super::Storage;
use alloc::sync::Arc;
use core::fmt;

/// A Storage wrapping a byte array in memory.
#[derive(Clone)]
pub struct MemoryStorage<B: Clone + AsRef<[u8]>>(B);

impl<B: Clone + AsRef<[u8]>> MemoryStorage<B> {
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
        Arc::new(Self(backing))
    }
}

impl<B: Clone + AsRef<[u8]>> Storage for MemoryStorage<B> {
    fn read_at(self: &Arc<Self>, offset: u64, buf: &mut [u8]) -> Result<usize, ()> {
        let inner = self.0.as_ref();
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
        self.0.as_ref().len() as _
    }
}

impl<B: Clone + AsRef<[u8]>> fmt::Debug for MemoryStorage<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MemoryBackend")
            .field(&self.0.as_ref())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{MemoryStorage, Storage};

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
}
