use crate::storage::{Storage, StorageMapper, SubStorage};
use crate::utils::string_table::StringTable;
use alloc::{sync::Arc, vec::Vec};
use binrw::BinRead;
use bstr::BStr;
use core::fmt;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[binrw::binrw]
#[brw(little)]
struct Pfs0Entry {
    offset: u64,
    size: u64,
    string_offset: u32,
    reserved: u32,
}

pub struct Entry<'a, S: ?Sized> {
    parent: &'a Pfs0<S>,
    data: Arc<SubStorage<SubStorage<S>>>,
    raw: Pfs0Entry,
}

impl<'a, S: ?Sized> Entry<'a, S> {
    pub fn name(&self) -> &BStr {
        self.parent
            .hdr
            .string_table
            .get(self.raw.string_offset as usize)
            .unwrap_or_default()
    }
}

impl<'a, S: ?Sized + Storage> Entry<'a, S> {
    pub fn data(&self) -> Arc<impl Storage> {
        Arc::clone(&self.data)
    }
}

impl<'a, S: ?Sized> fmt::Debug for Entry<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("name", &self.name())
            .field("raw", &self.raw)
            .finish()
    }
}

#[binrw::binrw]
#[brw(magic = b"PFS0", little)]
pub struct Pfs0Header {
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    entry_cnt: u32,

    #[br(temp)]
    #[bw(calc = string_table.0.len() as u32)]
    string_table_size: u32,

    reserved: u32,

    #[br(count = entry_cnt)]
    entries: Vec<Pfs0Entry>,

    #[br(args(string_table_size as usize))]
    string_table: StringTable,
}

impl Pfs0Header {
    const STATIC_HDR_SIZE: usize = 0x4 + 0x4 + 0x4 + 0x4;

    pub fn size(&self) -> usize {
        Self::STATIC_HDR_SIZE
            + self.entries.len() * core::mem::size_of::<Pfs0Entry>()
            + self.string_table.0.len()
    }
}

pub struct Pfs0<S: ?Sized> {
    hdr: Pfs0Header,
    data: Arc<SubStorage<S>>,
}

impl<S: ?Sized + Storage> Pfs0<S> {
    pub fn from_storage(parent: &Arc<S>) -> Self {
        let hdr = Pfs0Header::read(&mut parent.to_file_like()).unwrap();
        let data = parent.split(hdr.size() as u64, parent.length() - hdr.size() as u64);
        Self { hdr, data }
    }

    pub fn files(&self) -> impl Iterator<Item = Entry<'_, S>> {
        self.hdr.entries.iter().map(|e| Entry {
            parent: self,
            raw: e.clone(),
            data: self.data.split(e.offset, e.size),
        })
    }
}

impl<S: ?Sized + Storage> StorageMapper<S> for Pfs0<S> {
    type Options = ();
    type Output = Result<Self, ()>;

    fn map_from_storage(s: &Arc<S>, _: Self::Options) -> Self::Output {
        Ok(Self::from_storage(s))
    }
}
