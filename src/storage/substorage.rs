use super::Storage;
use alloc::sync::Arc;

pub struct SubStorage<S: ?Sized> {
    parent: Arc<S>,
    offset: u64,
    len: u64,
}

impl<S: Storage + ?Sized> SubStorage<S> {
    pub fn split_from(parent: Arc<S>, offset: u64, len: u64) -> Arc<Self> {
        Arc::new(Self {
            parent,
            offset,
            len,
        })
    }
}

impl<S: ?Sized + Storage> Storage for SubStorage<S> {
    fn read_at(&self, offset: u64, mut buf: &mut [u8]) -> Result<usize, ()> {
        use core::cmp::min;

        let buf_len = buf.len();
        let available_len = self.len.checked_sub(offset).unwrap_or(0);
        buf = &mut buf[..min(available_len as usize, buf_len)];

        if buf.len() == 0 {
            return Ok(0);
        }

        self.parent.read_at(self.offset + offset, buf)
    }

    fn length(&self) -> u64 {
        self.len
    }
}
