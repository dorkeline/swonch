use alloc::vec::Vec;
use bstr::{BStr, ByteSlice};

#[binrw::binrw]
#[br(import(size: usize))]
pub struct StringTable(#[br(count = size)] pub Vec<u8>);

impl StringTable {
    pub fn get(&self, index: usize) -> Option<&BStr> {
        self.0.get(index..).and_then(|start| {
            start
                .iter()
                .position(|c| *c == 0)
                .map(|end| start[..end].as_bstr())
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &BStr> {
        StringTableIter {
            table: self,
            offset: 0,
        }
    }
}

struct StringTableIter<'a> {
    table: &'a StringTable,
    offset: usize,
}

impl<'a> Iterator for StringTableIter<'a> {
    type Item = &'a BStr;

    fn next(&mut self) -> Option<Self::Item> {
        self.table.get(self.offset).map(|s| {
            self.offset += s.len() + 1;
            s
        })
    }
}
