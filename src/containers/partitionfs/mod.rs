pub mod hfs0;
pub mod pfs0;

use crate::{
    storage::{mapper::FromStorage, IStorage, Storage},
    utils::{sealed::Sealed, string_table::StringTable},
    SwonchResult,
};

use alloc::vec::Vec;
use binrw::meta::ReadEndian;
use binrw::{BinRead, BinWrite};
use bstr::BStr;
use core::fmt;

pub struct Entry<'a, H: HeaderLike> {
    parent: &'a PartitionFs<H>,
    raw: H::RawEntry,
}

pub trait EntryLike: Sealed + fmt::Debug + Clone {
    fn string_offset(&self) -> u32;
    fn size(&self) -> u64;
    fn offset(&self) -> u64;
}

impl<'a, H: HeaderLike> Entry<'a, H> {
    pub fn name(&self) -> &BStr {
        self.parent
            .hdr
            .string_table()
            .get(self.raw.string_offset() as usize)
            .unwrap_or_default()
    }

    pub fn data(&self) -> SwonchResult<Storage> {
        // if this fails either the storage changed its mind about having a length (BUG)
        // or the offset/size reported is incorrect which is a corrupt/malicious PFS0.
        Ok(self
            .parent
            .data
            .clone()
            .split(self.raw.offset(), self.raw.size())?)
    }
}

impl<'a, H: HeaderLike> fmt::Debug for Entry<'a, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("name", &self.name())
            .field("raw", &self.raw)
            .finish()
    }
}

#[binrw::binrw]
#[brw(little)]
struct CommonHeader<E>
where
    E: BinRead + BinWrite + 'static,
    E: for<'a> BinRead<Args<'a> = ()>,
    E: for<'a> BinWrite<Args<'a> = ()>,
{
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    entry_cnt: u32,

    #[br(temp)]
    #[bw(calc = string_table.as_bytes().len() as u32)]
    string_table_size: u32,

    reserved: u32,

    #[br(count = entry_cnt)]
    entries: Vec<E>,

    #[br(args(string_table_size as usize))]
    string_table: StringTable,
}

pub trait HeaderLike: Sealed
where
    Self: BinRead + BinWrite + ReadEndian + 'static,
    Self: for<'a> BinRead<Args<'a> = ()>,
    Self: for<'a> BinWrite<Args<'a> = ()>,
{
    type RawEntry: EntryLike;

    fn size(&self) -> usize;

    fn entries(&self) -> &Vec<Self::RawEntry>;

    fn string_table(&self) -> &StringTable;
}

pub struct PartitionFs<H: BinRead + BinWrite> {
    hdr: H,
    data: Storage,
}

impl<H: HeaderLike> PartitionFs<H> {
    pub fn from_storage(parent: Storage) -> SwonchResult<Self> {
        let hdr = H::read(&mut parent.clone().into_stdio())?;
        let data = parent
            .clone()
            .split(hdr.size() as u64, parent.length()? - hdr.size() as u64)?;
        Ok(Self { hdr, data })
    }

    pub fn names(&self) -> impl Iterator<Item = &BStr> {
        self.hdr.string_table().iter().filter(|n| !n.is_empty())
    }

    pub fn files(&self) -> impl Iterator<Item = Entry<'_, H>> {
        self.hdr.entries().iter().map(|e| {
            Entry {
                parent: self,
                raw: e.clone(),
            }
        })
    }
}

impl<H: HeaderLike> FromStorage for PartitionFs<H> {
    type Args = ();

    fn from_storage(parent: Storage, _: Self::Args) -> SwonchResult<Self> {
        Ok(PartitionFs::from_storage(parent)?)
    }
}
