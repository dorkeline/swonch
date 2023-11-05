use super::{IStorage, Storage};
use binrw::io::{Read, Seek, SeekFrom};

/// wrap a Storage to get a type providing Read/Seek/Write implementations
pub struct StorageStdioWrapper {
    s: Storage,
    offset: u64,
}

impl StorageStdioWrapper {
    pub fn new(s: Storage) -> Self {
        Self { s, offset: 0 }
    }
}

impl Read for StorageStdioWrapper {
    fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
        self.s
            .read_at(self.offset, buf)
            .map(|size| {
                self.offset += size as u64;
                size as _
            })
            .map_err(crate::utils::other_io_error)
    }
}

impl Seek for StorageStdioWrapper {
    fn seek(&mut self, pos: SeekFrom) -> binrw::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.offset = offset,
            SeekFrom::Current(off) => {
                self.offset = (self.offset as i64 + off) as u64;
            }
            SeekFrom::End(off) => {
                self.offset = (self.s.length()? as i64 - off) as u64;
            }
        };

        Ok(self.offset)
    }
}
