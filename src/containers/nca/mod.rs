use crate::{
    prelude::IStorage,
    storage::{FromStorage, Storage},
    SwonchResult,
};

pub mod header;
pub use header::*;

pub struct Nca {}

impl Nca {
    pub fn sections(&self) -> impl Iterator<Item = ()> {
        core::iter::empty()
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub enum NcaError {
    #[error("header is not in plaintext but no header_key was given")]
    NoKeyGivenForEncryptedHeader,

    #[error("header seems to be corrupted")]
    HeaderCorrupted,
}

impl FromStorage for Nca {
    type Args = Option<[u8; 0x20]>;

    fn from_storage(parent: Storage, header_key: Self::Args) -> SwonchResult<Self> {
        let mut nca_enc_area = vec![0; 0xc00];
        let read_cnt = parent.read_at(0x00, &mut nca_enc_area)?;

        let hdr = NcaHeader::from_buf(&mut nca_enc_area[..read_cnt as usize], header_key)?;

        //println!("{:#?}", hdr);

        todo!()
    }
}
