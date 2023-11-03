use super::Storage;

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    fn read_at(self: &Arc<Self>, offset: u64, buf: &mut [u8]) -> Result<usize, ()> {
        let mut fp = self.fp.lock().unwrap();

        fp.seek(SeekFrom::Start(offset)).unwrap();
        let cnt = fp.read(buf).unwrap();

        Ok(cnt)
    }

    fn length(&self) -> u64 {
        self.fp.lock().unwrap().metadata().unwrap().len()
    }
}
