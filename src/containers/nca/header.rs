use crate::{keyset::KEYS, SwonchResult};
use binrw::{io::Cursor, BinRead};
use core::fmt;
use xts_mode::Xts128;

use super::NcaError;
use crate::{
    common::{ProgramId, RightsId},
    utils::{self, HexArray},
};

#[binrw::binrw]
#[brw(little, magic = b"NCA")]
#[derive(Debug, Copy, Clone)]
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

impl From<NcaMagic> for u8 {
    fn from(value: NcaMagic) -> Self {
        use NcaMagic::*;
        match value {
            Nca0 => 0,
            Nca1 => 1,
            Nca2 => 2,
            Nca3 => 3,
            Unknown(i) => i,
        }
    }
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

impl fmt::Display for KeyAreaEncryptionKeyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                KeyAreaEncryptionKeyIndex::Application => "application",
                KeyAreaEncryptionKeyIndex::Ocean => "ocean",
                KeyAreaEncryptionKeyIndex::System => "system",
            }
        )
    }
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct NcaFsEntry {
    // block size == 0x200
    pub start_offset_block: u32,
    pub end_offset_block: u32,
    pub flags: u32,
    pub reserved1: u32,
}

impl NcaFsEntry {
    pub fn is_active(&self) -> bool {
        self.flags & 1 == 1
    }
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
    pub fixed_key_hdr_signature: HexArray<0x100>,
    pub npdm_hdr_signature: HexArray<0x100>,
    pub magic: NcaMagic,
    pub distribution_type: DistributionType,
    pub content_type: ContentType,
    pub key_generation_old: u8,
    pub key_area_encryption_key_index: KeyAreaEncryptionKeyIndex,
    pub content_size: u64,
    pub program_id: ProgramId,
    pub content_index: u32,
    pub sdk_addon_version: SdkAddonVersion,
    pub key_generation: u8,
    pub signature_key_generation: u8, // 9.0.0+
    pub reserved: HexArray<0xe>,
    pub rights_id: RightsId,
    pub fs_entries: [NcaFsEntry; 4],
    pub fs_entry_hashes: [HexArray<0x20>; 4],
    pub encrypted_key_area: [HexArray<0x10>; 4],
}

impl NcaHeader {
    pub fn from_buf(mut buf: &mut [u8]) -> SwonchResult<Self> {
        if NcaHeader::is_encrypted(&buf) {
            let xts: Xts128<_> = KEYS
                .get_key::<crate::keyset::Aes128XtsKey>("header_key")?
                .into();

            xts.decrypt_area(&mut buf, 0x200, 0, utils::aes_xtsn_tweak);
        }

        //crate::utils::dbg_hexdump(std::io::stdout(), buf);

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

    pub(crate) fn get_key_generation_index(&self) -> u8 {
        core::cmp::max(self.key_generation, self.key_generation_old).saturating_sub(1)
    }
}
