use aes::Aes128;
use alloc::sync::Arc;

use crate::{keyset::KEYS, prelude::*, utils::HexArray, SwonchResult};

use super::Nca;

#[binrw::binrw]
#[brw(little, repr(u8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsType {
    RomFS = 0,
    PartitionFS = 1,
}

#[binrw::binrw]
#[brw(little, repr(u8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    Auto = 0,
    None = 1,
    AesXts = 2,
    AesCtr = 3,
    AesCtrEx = 4,
    AesCtrSkipLayerHash = 5,   // 14.0.0+
    AesCtrExSkipLayerHash = 6, // 14.0.0+
}

#[binrw::binrw]
#[brw(little, repr(u8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    Auto = 0,
    None = 1,
    HierarchicalSha256Hash = 2,
    HierarchicalIntegrityHash = 3,
    AutoSha3 = 4,                      // 14.0.0+
    HierarchicalSha3256Hash = 5,       // 14.0.0+
    HierarchicalIntegritySha3Hash = 6, // 14.0.0+
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct FsHeader {
    pub version: u16,
    pub fs_type: FsType,
    pub hash_type: HashType,
    pub encryption_type: EncryptionType,
    metadata_hash_type: u8,
    reserved0: HexArray<2>,
    hash_data: HexArray<0xf8>,
    patch_info: HexArray<0x40>,
    generation: u32,
    secure_value: u32,
    sparse_info: HexArray<0x30>,
    compression_info: HexArray<0x28>,   // 12.0.0+
    metadata_hash_info: HexArray<0x30>, // 14.0.0+
    reserved1: HexArray<0x30>,
}

pub struct NcaSection {
    pub(crate) parent: Arc<Nca>,
    pub(crate) fs_header: Arc<FsHeader>,
    pub(crate) index: u32,
}

impl NcaSection {
    pub fn header(&self) -> &FsHeader {
        &self.fs_header
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn open_encrypted(&self) -> SwonchResult<Storage> {
        let fs_entry = &self.parent.header.fs_entries[self.index as usize];

        self.parent.storage.clone().split(
            fs_entry.start_offset_block as u64 * 0x200,
            (fs_entry.end_offset_block - fs_entry.start_offset_block) as u64 * 0x200,
        )
    }

    fn get_key_for_tkey_crypto(&self) -> SwonchResult<[u8; 0x10]> {
        let rights_id = self.parent.header.rights_id;
        let tkey_enc = KEYS.get_titlekey(rights_id)?;
        let key_generation = self.parent.header.get_key_generation_index();
        let tkey = crate::utils::decrypt_titlekey(tkey_enc, key_generation)?;

        Ok(tkey)
    }

    pub(crate) fn get_key_for_section_decryption(&self) -> SwonchResult<Option<[u8; 0x10]>> {
        if self.fs_header.encryption_type == EncryptionType::None {
            return Ok(None);
        }

        let rights_id = self.parent.header.rights_id;
        let key_generation = self.parent.header.get_key_generation_index();

        Some(if rights_id.0 == 0 {
            // default crypto
            todo!()
        } else {
            self.get_key_for_tkey_crypto()
        })
        .transpose()
    }

    pub fn open_decrypted(&self) -> SwonchResult<Storage> {
        let rights_id = self.parent.header.rights_id;
        // let key_area_key = KEYS.get_key_index(
        //    format!(
        //       "key_area_key_{}",
        //      self.parent.header.key_area_encryption_key_index
        // ),
        // key_generation,
        //);
        let section_data = self.open_encrypted()?;
        let key = self.get_key_for_section_decryption()?;

        use EncryptionType::*;
        Ok(match self.fs_header.encryption_type {
            None => section_data,
            Auto => unimplemented!(),
            AesCtr => {
                use crate::storage::crypto::AesCtrStorage;
                use aes::cipher::{KeyInit, KeyIvInit};
                use ctr::Ctr64LE;

                let iv = [0u8; 0x10];
                let aes_ctx = Ctr64LE::<Aes128>::new_from_slices(&key.unwrap(), &iv).unwrap();

                AesCtrStorage::new(section_data, aes_ctx).into_storage()
            }
            _ => todo!(),
        })
    }
}
