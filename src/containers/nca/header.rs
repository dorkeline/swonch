use crate::SwonchResult;
use binrw::{io::Cursor, BinRead};
use core::fmt;

use super::NcaError;
use crate::{
    common::{ProgramId, RightsId},
    utils::{self, HexArray},
};

#[binrw::binrw]
#[brw(little, magic = b"NCA")]
#[derive(Debug, Clone)]
pub enum NcaMagic {
    #[brw(magic = b'0')]
    Nca0,
    #[brw(magic = b'1')]
    Nca1,
    #[brw(magic = b'2')]
    Nca2,
    #[brw(magic = b'3')]
    Nca3,
    Unknown(u8),
}

#[binrw::binrw]
#[brw(little, repr(u8))]
#[derive(Debug, Clone)]
pub enum DistributionType {
    Download = 0x0,
    GameCard = 0x1,
}

#[binrw::binrw]
#[brw(little, repr(u8))]
#[derive(Debug, Clone)]
pub enum ContentType {
    Program = 0x0,
    Meta = 0x1,
    Control = 0x2,
    Manual = 0x3,
    Data = 0x4,
    PublicData = 0x5,
}

#[binrw::binrw]
#[brw(little, repr(u8))]
#[derive(Debug, Clone)]
pub enum KeyAreaEncryptionKeyIndex {
    Application = 0x0,
    Ocean = 0x1,
    System = 0x2,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct NcaFsEntry {
    // block size == 0x200
    start_offset_block: u32,
    end_offset_block: u32,
    reserved: u64,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct SdkAddonVersion {
    _zero: u8,
    micro: u8,
    minor: u8,
    major: u8,
}

impl fmt::Display for SdkAddonVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.micro, self._zero
        )
    }
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct NcaHeader {
    fixed_key_hdr_signature: HexArray<0x100>,
    npdm_hdr_signature: HexArray<0x100>,
    magic: NcaMagic,
    distribution_type: DistributionType,
    content_type: ContentType,
    key_generation_old: u8,
    key_area_encryption_key_index: KeyAreaEncryptionKeyIndex,
    content_size: u64,
    program_id: ProgramId,
    content_index: u32,
    sdk_addon_version: SdkAddonVersion,
    key_generation: u8,
    signature_key_generation: u8, // 9.0.0+
    reserved: HexArray<0xe>,
    rights_id: RightsId,
    fs_entries: [NcaFsEntry; 4],
    fs_entry_hashes: [HexArray<0x20>; 4],
    encrypted_key_area: [HexArray<0x10>; 4],
}

impl NcaHeader {
    pub fn from_buf(mut buf: &mut [u8], header_key: Option<[u8; 0x20]>) -> SwonchResult<Self> {
        if NcaHeader::is_encrypted(&buf) {
            let Some(header_key) = header_key else {
                return Err(NcaError::NoKeyGivenForEncryptedHeader.into());
            };

            use aes::{
                cipher::{generic_array::GenericArray, KeyInit},
                Aes128,
            };
            use xts_mode::Xts128;

            let (c1, c2) = (&header_key[..0x10], &header_key[0x10..]);
            let (crypt, tweak) = (
                Aes128::new(GenericArray::from_slice(c1)),
                Aes128::new(GenericArray::from_slice(c2)),
            );

            let xts = Xts128::new(crypt, tweak);

            xts.decrypt_area(&mut buf, 0x200, 0, utils::aes_xtsn_tweak);
        }

        if NcaHeader::is_encrypted(&buf) {
            return Err(NcaError::HeaderCorrupted.into());
        }

        NcaHeader::read(&mut Cursor::new(buf)).map_err(Into::into)
    }

    pub(crate) fn is_encrypted(buf: &[u8]) -> bool {
        match buf[0x200..][..4] {
            [b'N', b'C', b'A', b'3' | b'2' | b'1' | b'0'] => false,
            _ => true,
        }
    }
}
