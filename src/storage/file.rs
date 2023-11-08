use super::{IStorage, Storage, SwonchResult};

use crate::sync_impl::Mutex;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug)]
pub struct FileStorage {
    fp: Mutex<File>,
}

impl FileStorage {
    pub fn new(fp: File) -> Storage {
        Storage::new(Self { fp: Mutex::new(fp) })
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Storage> {
        File::open(path).map(Self::new)
    }
}

impl IStorage for FileStorage {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<u64> {
        let mut fp = self.fp.lock();

        fp.seek(SeekFrom::Start(offset))?;
        let cnt = fp.read(buf)?;

        Ok(cnt as _)
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> SwonchResult<u64> {
        let mut fp = self.fp.lock();
        fp.seek(SeekFrom::Start(offset))?;
        let cnt = fp.write(data)?;
        Ok(cnt as _)
    }

    fn is_readonly(&self) -> bool {
        self.fp
            .lock()
            .metadata()
            .map(|m| m.permissions().readonly())
            .unwrap_or(true)
    }

    fn length(&self) -> SwonchResult<u64> {
        Ok(self.fp.lock().metadata()?.len())
    }
}
