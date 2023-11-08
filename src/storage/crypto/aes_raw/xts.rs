use crate::{
    storage::{IStorage, Storage},
    SwonchResult,
};
use alloc::sync::Arc;
use core::{fmt, marker::PhantomData};

use aes::Aes128;
use xts_mode::Xts128;

pub trait Tweak: fmt::Debug + Clone + 'static {
    fn get_tweak(sector: u128) -> [u8; 0x10];
}

#[derive(Debug, Clone)]
pub struct DefaultTweak;
impl Tweak for DefaultTweak {
    fn get_tweak(sector: u128) -> [u8; 0x10] {
        xts_mode::get_tweak_default(sector)
    }
}

#[derive(Debug, Clone)]
pub struct NintendoTweak;
impl Tweak for NintendoTweak {
    fn get_tweak(sector: u128) -> [u8; 0x10] {
        crate::utils::aes_xtsn_tweak(sector)
    }
}

#[derive(Clone)]
pub struct AesXtsStorageImpl<T: Tweak = DefaultTweak> {
    parent: Storage,
    aes_ctx: Arc<Xts128<Aes128>>,
    // add to sectors before calculating tweaks
    sector_offset: i64,
    tweak: PhantomData<T>,
}

impl<T: Tweak> AesXtsStorageImpl<T> {
    pub fn new(s: Storage, key: Xts128<Aes128>, sector_offset: i64) -> Self {
        Self {
            parent: s,
            aes_ctx: Arc::new(key),
            sector_offset,
            tweak: PhantomData,
        }
    }

    pub fn set_sector_offset(&mut self, new: i64) {
        self.sector_offset = new
    }
}

impl<T: Tweak> fmt::Debug for AesXtsStorageImpl<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AesXtsStorage")
            .field("parent", &self.parent)
            .field("sector_offset", &self.sector_offset)
            .finish_non_exhaustive()
    }
}

impl<T: Tweak> AesXtsStorageImpl<T> {
    pub const SECTOR_SIZE: u64 = 0x200;
}

impl<T: fmt::Debug + Tweak> IStorage for AesXtsStorageImpl<T> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<u64> {
        assert!(
            offset % Self::SECTOR_SIZE == 0 && buf.len() % Self::SECTOR_SIZE as usize == 0,
            "unaligned read access to AesXtsStorage. If you dont want to manually align offsets and lengths yourself please use BlockBufferStorage<AesXtsStorage> instead"
        );

        let cnt = self.parent.read_at(offset, buf)?;
        let sector = (offset / Self::SECTOR_SIZE) as i64 + self.sector_offset;
        self.aes_ctx.decrypt_area(
            buf,
            Self::SECTOR_SIZE as usize,
            sector as u128,
            T::get_tweak,
        );

        Ok(cnt)
    }

    fn is_readonly(&self) -> bool {
        true
    }

    fn length(&self) -> SwonchResult<u64> {
        self.parent.length()
    }
}

pub type AesXtsStorage = AesXtsStorageImpl<DefaultTweak>;
pub type AesXtsnStorage = AesXtsStorageImpl<NintendoTweak>;

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::{generic_array::GenericArray, KeyInit};

    fn _key() -> (Aes128, Aes128) {
        let crypt = Aes128::new(GenericArray::from_slice(b"cafebabecafebabe"));
        let tweak = Aes128::new(GenericArray::from_slice(b"cafebabecafebabe"));
        (crypt, tweak)
    }

    fn _encrypt(buf: &mut [u8]) {
        let (crypt, tweak) = _key();
        let xts = Xts128::new(crypt, tweak);

        xts.encrypt_area(buf, 0x200, 0, xts_mode::get_tweak_default);
    }
}
