use super::{Storage, SwonchResult};

use crate::sync_impl::Mutex;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

pub struct FileStorage {
    fp: Mutex<File>,
}

impl FileStorage {
    pub fn new(fp: File) -> Arc<Self> {
        Arc::new(Self { fp: Mutex::new(fp) })
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Arc<Self>> {
        File::open(path).map(Self::new)
    }
}

impl Storage for FileStorage {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> SwonchResult<usize> {
        let mut fp = self.fp.lock();

        fp.seek(SeekFrom::Start(offset))?;
        let cnt = fp.read(buf)?;

        Ok(cnt)
    }

    fn length(&self) -> u64 {
        self.fp.lock().metadata().expect("wat").len()
    }
}
