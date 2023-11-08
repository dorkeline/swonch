use crate::{
    keyset::{KeyError, KEYS},
    prelude::IStorage,
    storage::{FromStorage, Storage},
    utils, SwonchResult,
};
use alloc::{sync::Arc, vec::Vec};

use aes::{
    cipher::{generic_array::GenericArray, KeyInit},
    Aes128,
};
use binrw::{io::Cursor, BinRead};
use xts_mode::Xts128;

pub mod header;
pub use header::*;
pub mod section;
pub use section::*;

#[derive(Debug)]
pub struct Nca {
    storage: Storage,
    header: NcaHeader,
    fs_headers: [Option<Arc<FsHeader>>; 4],
}

impl Nca {
    pub fn header(&self) -> &NcaHeader {
        &self.header
    }

    pub fn sections(self: &Arc<Self>) -> impl Iterator<Item = NcaSection> {
        // collect is needed because the iterator captures a lifetime otherwise
        let sections = self
            .fs_headers
            .iter()
            .enumerate()
            .flat_map(|(idx, hdr)| hdr.as_ref().map(|hdr| (idx, hdr)))
            .map(|(index, fs_hdr)| NcaSection {
                parent: self.clone(),
                fs_header: fs_hdr.clone(),
                index: index as u32,
            })
            .collect::<Vec<_>>();

        sections.into_iter()
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
    type Args = ();
    type Output = SwonchResult<Arc<Self>>;

    fn from_storage(parent: Storage, _: Self::Args) -> Self::Output {
        let mut buf = vec![0; 0xc00];
        let read_cnt = parent.read_at(0, &mut buf)?;

        let hdr = NcaHeader::from_buf(&mut buf[..0x400 as usize])?;

        let fs_header_area = &mut buf[0x400..][..0x200 * 4];
        let mut fs_headers = [None, None, None, None];

        for idx in 0..4 {
            if !hdr.fs_entries.get(idx).unwrap().is_active() {
                continue;
            }

            let fs_header = &mut fs_header_area[0x200 * idx..][..0x200];

            // FsHeader.version is apparently always 2, check that to see whether we're encrypted
            if u32::from_le_bytes(fs_header[..4].try_into().unwrap()) != 2 {
                // pre 1.0.0 (NCA3) ncas reset the sector index for each fs header
                // TODO: test this, as i dont have a NCA2 or earlier nca
                let nca_ver: u8 = hdr.magic.into();
                let sector = match nca_ver {
                    3.. => 2 + idx as u128,
                    ..=2 => {
                        log::warn!("NCA2 and earlier is untested, if this works or causes issues id love to hear some feedback!");
                        0
                    }
                };

                let xts = KEYS
                    .get_key::<crate::keyset::Aes128XtsKey>("header_key")
                    .map(Into::<Xts128<_>>::into)?;

                xts.decrypt_sector(fs_header, utils::aes_xtsn_tweak(sector))
            }

            let fs_hdr = FsHeader::read(&mut Cursor::new(&fs_header))?;

            fs_headers[idx] = Some(Arc::new(fs_hdr));

            //utils::dbg_hexdump(std::io::stdout(), &fs_header);
        }

        //log::debug!("{hdr:?}");

        Ok(Arc::new(Self {
            storage: parent,
            header: hdr,
            fs_headers,
        }))
    }
}
