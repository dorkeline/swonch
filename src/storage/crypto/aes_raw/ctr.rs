use crate::{prelude::*, sync_impl::Mutex};
use aes::{
    cipher::{StreamCipher, StreamCipherSeek},
    Aes128,
};
use ctr::Ctr64LE;

use alloc::{sync::Arc, vec::Vec};
use core::fmt;

const BUF_SIZE: usize = 1024 * 1024;

#[derive(Clone)]
pub struct AesCtrStorageImpl {
    parent: Storage,
    aes_ctx: Arc<Mutex<Ctr64LE<Aes128>>>,
    write_buf: Arc<Mutex<Vec<u8>>>,
}

impl AesCtrStorageImpl {
    pub fn new(parent: Storage, aes_ctx: Ctr64LE<Aes128>) -> Self {
        Self {
            parent,
            aes_ctx: Arc::new(Mutex::new(aes_ctx)),
            write_buf: Arc::new(Mutex::new(vec![0; BUF_SIZE])),
        }
    }
}

impl fmt::Debug for AesCtrStorageImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AesCtrStorageImpl")
            .field("parent", &self.parent)
            .finish_non_exhaustive()
    }
}

impl IStorage for AesCtrStorageImpl {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<u64> {
        let mut aes = self.aes_ctx.lock();
        aes.seek(offset);
        let len = self.parent.read_at(offset, buf)?;
        aes.apply_keystream(&mut buf[..len as usize]);
        Ok(len)
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> SwonchResult<u64> {
        if self.is_readonly() {
            return Err(SwonchError::StorageIsReadOnly);
        }

        let mut aes = self.aes_ctx.lock();
        let buf = &mut *self.write_buf.lock();

        let mut cnt = 0;
        for chunk in data.chunks(BUF_SIZE) {
            aes.apply_keystream_b2b(chunk, buf).unwrap();
            cnt += self.parent.write_at(offset + cnt, buf)?;
        }

        Ok(cnt)
    }

    fn length(&self) -> SwonchResult<u64> {
        self.parent.length()
    }

    fn is_readonly(&self) -> bool {
        self.parent.is_readonly()
    }
}
