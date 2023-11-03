pub mod hfs0;
pub mod pfs0;

use crate::storage::{Storage, StorageMapper, SubStorage};
use crate::utils::sealed::Sealed;
use crate::utils::string_table::StringTable;
use alloc::{sync::Arc, vec::Vec};
use binrw::meta::ReadEndian;
use binrw::{BinRead, BinWrite};
use bstr::BStr;
use core::fmt;

pub struct Entry<'a, S: ?Sized + Storage, H: HeaderLike> {
    parent: &'a PartitionFs<H, S>,
    data: Arc<SubStorage<SubStorage<S>>>,
    raw: H::RawEntry,
}

pub trait EntryLike: Sealed + fmt::Debug + Clone {
    fn string_offset(&self) -> u32;
    fn size(&self) -> u64;
    fn offset(&self) -> u64;
}

impl<'a, S: ?Sized + Storage, H: HeaderLike> Entry<'a, S, H> {
    pub fn name(&self) -> &BStr {
        self.parent
            .hdr
            .string_table()
            .get(self.raw.string_offset() as usize)
            .unwrap_or_default()
    }

    pub fn data(&self) -> Arc<impl Storage> {
        Arc::clone(&self.data)
    }
}

impl<'a, S: ?Sized + Storage, H: HeaderLike> fmt::Debug for Entry<'a, S, H> {
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

pub struct PartitionFs<H: BinRead + BinWrite, S: ?Sized + Storage> {
    hdr: H,
    data: Arc<SubStorage<S>>,
}

impl<H: HeaderLike, S: ?Sized + Storage> PartitionFs<H, S> {
    pub fn from_storage(parent: &Arc<S>) -> Self {
        let hdr = H::read(&mut parent.to_file_like()).unwrap();
        let data = parent.split(hdr.size() as u64, parent.length() - hdr.size() as u64);
        Self { hdr, data }
    }

    pub fn names(&self) -> impl Iterator<Item = &BStr> {
        self.hdr.string_table().iter().filter(|n| !n.is_empty())
    }

    pub fn files(&self) -> impl Iterator<Item = Entry<'_, S, H>> {
        self.hdr.entries().iter().map(|e| Entry {
            parent: self,
            raw: e.clone(),
            data: self.data.split(e.offset(), e.size()),
        })
    }
}

impl<H: HeaderLike, S: ?Sized + Storage> StorageMapper<S> for PartitionFs<H, S> {
    type Options = ();
    type Output = Result<Self, ()>;

    fn map_from_storage(s: &Arc<S>, _: Self::Options) -> Self::Output {
        Ok(Self::from_storage(s))
    }
}
