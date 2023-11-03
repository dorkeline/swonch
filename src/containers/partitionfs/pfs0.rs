use crate::utils::sealed::Sealed;
use crate::utils::string_table::StringTable;
use alloc::vec::Vec;

use super::{CommonHeader, EntryLike, HeaderLike};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[binrw::binrw]
#[brw(little)]
pub struct RawPfs0Entry {
    offset: u64,
    size: u64,
    string_offset: u32,
    reserved: u32,
}

impl Sealed for RawPfs0Entry {}
impl EntryLike for RawPfs0Entry {
    fn size(&self) -> u64 {
        self.size
    }

    fn offset(&self) -> u64 {
        self.offset
    }

    fn string_offset(&self) -> u32 {
        self.string_offset
    }
}

#[binrw::binrw]
#[brw(magic = b"PFS0", little)]
pub struct Pfs0Header(CommonHeader<RawPfs0Entry>);

impl Pfs0Header {
    const STATIC_HDR_SIZE: usize = 0x4 + 0x4 + 0x4 + 0x4;

    pub fn size(&self) -> usize {
        Self::STATIC_HDR_SIZE
            + self.0.entries.len() * core::mem::size_of::<RawPfs0Entry>()
            + self.0.string_table.as_bytes().len()
    }
}

impl Sealed for Pfs0Header {}
impl HeaderLike for Pfs0Header {
    type RawEntry = RawPfs0Entry;

    fn entries(&self) -> &Vec<Self::RawEntry> {
        &self.0.entries
    }

    fn size(&self) -> usize {
        Pfs0Header::size(self)
    }

    fn string_table(&self) -> &StringTable {
        &self.0.string_table
    }
}

pub type Pfs0<S> = super::PartitionFs<Pfs0Header, S>;
