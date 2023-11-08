pub mod aes_xts;

use crate::{sync_impl::Mutex, SwonchResult};
use alloc::boxed::Box;
use super::{FromStorage, IStorage};

/// A buffered and self aligning wrapper storage for AES128 in XTS mode. 
pub type AesXtsStorage = BlockBufferStorage<aes_xts::AesXtsStorage, 0x200>;

/// A buffered and self aligning wrapper storage for AES128 in XTS mode with Nintendo's custom tweak. 
pub type AesXtsnStorage = BlockBufferStorage<aes_xts::AesXtsnStorage, 0x200>;


#[derive(Debug)]
pub struct BlockBufferStorage<S: IStorage, const N: usize> {
    inner: S,
    cache: Mutex<(Option<u64>, Box<[u8; N]>)>,
}

impl<S: IStorage, const N: usize> BlockBufferStorage<S, N> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            cache: Mutex::new((None, Box::new([0; N]))),
        }
    }

    fn get_aligned(
        mut offset: u64,
        mut len: u64,
    ) -> (
        Option<UnalignedSector>,
        Option<AlignedSector>,
        Option<UnalignedSector>,
    ) {
        let offset_sector_delta = offset % N as u64;
        let mut offset_into_out_buf = 0;

        let leading = {
            match (offset_sector_delta == 0, len) {
                // well aligned and read doesnt need to be padded, nothing to do
                (true, len) if len >= N as u64 => None,

                // start well aligned but read needs to be padded
                (true, len_into_out_buf) => {
                    len = 0;
                    Some(UnalignedSector {
                        aligned_start: offset,
                        offset_into_out_buf: 0,
                        offset_in_sector_buf: 0,
                        len_into_out_buf: len_into_out_buf as usize,
                    })
                }

                // absolutely ~~proprietary~~ unaligned
                (false, len_) => {
                    let aligned_start = offset - offset_sector_delta;
                    offset = aligned_start + N as u64;

                    let len_into_out_buf = match offset_sector_delta + len_ >= N as u64 {
                        true => N as u64 - offset_sector_delta,
                        false => len_,
                    } as usize;
                    len -= len_into_out_buf as u64;

                    offset_into_out_buf += len_into_out_buf;

                    Some(UnalignedSector {
                        aligned_start,
                        offset_into_out_buf: 0,
                        offset_in_sector_buf: offset_sector_delta as usize,
                        len_into_out_buf,
                    })
                }
            }
        };

        // if the start was unaligned leading takes care of that, consume as many full sectors as there are
        let full_sec_cnt = len / N as u64;
        let full_sec_len = full_sec_cnt * N as u64;
        len -= full_sec_len;
        let aligned = match full_sec_len {
            0 => None,
            n => Some(AlignedSector {
                start: offset,
                len: n,
                offset_into_out_buf: offset_into_out_buf as usize,
            }),
        };
        offset += full_sec_len;
        offset_into_out_buf += full_sec_len as usize;

        // if theres still data left, make another unaligned sector
        let trailing = match len {
            0 => None,
            n => Some(UnalignedSector {
                aligned_start: offset,
                offset_in_sector_buf: 0,
                offset_into_out_buf: offset_into_out_buf as usize,
                len_into_out_buf: n as usize,
            }),
        };

        (leading, aligned, trailing)
    }

    fn cache_aligned_single_sector_read(
        &self,
        offset: u64,
        offset_in_sector_buf: usize,
        buf: &mut [u8],
    ) -> SwonchResult<()> {
        let (ref mut cached_offset, ref mut cache) = &mut *self.cache.lock();

        match cached_offset {
            Some(off) if *off == offset => {
                // awesome, a cache hit!, dont do anything
            }
            _ => {
                // its not cached, read from upstream and update cache
                self.inner.read_at(offset, &mut cache[..N])?;
                *cached_offset = Some(offset);
            }
        }

        Ok(buf.copy_from_slice(&cache[offset_in_sector_buf..N]))
    }
}

// represents an unaligned access that had to be padded/broken up
// len is always a single sector, we never pad more than that
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct UnalignedSector {
    /// aligned start to pass to parent
    aligned_start: u64,

    /// offset into the parent buf at which to start copying into
    offset_into_out_buf: usize,

    /// offset into the sector buf from which to start copying
    offset_in_sector_buf: usize,

    /// length of the read sector to copy to parent buf
    len_into_out_buf: usize,
}

// represents a range of well aligned reads
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct AlignedSector {
    start: u64,
    len: u64,
    offset_into_out_buf: usize,
}

impl<S: IStorage, const N: usize> IStorage for BlockBufferStorage<S, N> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> crate::SwonchResult<u64> {
        let (leading, aligned, trailing) = Self::get_aligned(offset, buf.len() as u64);

        if let Some(leading) = leading {
            let buf = &mut buf[leading.offset_into_out_buf..][..leading.len_into_out_buf];
            self.cache_aligned_single_sector_read(
                leading.aligned_start,
                leading.offset_in_sector_buf,
                buf,
            )?;
        }

        if let Some(aligned) = aligned {
            let buf = &mut buf[aligned.offset_into_out_buf..][..aligned.len as usize];

            // reading a single aligned sector
            if aligned.len == N as u64 && trailing.is_none() {
                self.cache_aligned_single_sector_read(aligned.start, 0, buf)?;
            } else {
                // dont bother with caching on large, sequential reads
                self.inner.read_at(aligned.start, buf)?;
            }
        }

        if let Some(trailing) = trailing {
            let buf = &mut buf[trailing.offset_into_out_buf..][..trailing.len_into_out_buf];

            self.cache_aligned_single_sector_read(
                trailing.aligned_start,
                trailing.offset_in_sector_buf,
                buf,
            )?;
        }

        todo!()
    }

    fn write_at(&self, _offset: u64, _data: &[u8]) -> crate::SwonchResult<u64> {
        todo!()
    }

    fn is_readonly(&self) -> bool {
        //self.inner.is_readonly()
        true
    }

    fn length(&self) -> crate::SwonchResult<u64> {
        self.inner.length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_alignment_test() {
        type B = BlockBufferStorage<aes_xts::AesXtsStorage, 0x200>;
        let get_aligned = B::get_aligned;

        assert_eq!(
            get_aligned(0, 0x200),
            (
                None,
                Some(AlignedSector {
                    start: 0,
                    len: 0x200,
                    offset_into_out_buf: 0
                }),
                None
            )
        );

        assert_eq!(
            get_aligned(0, 0x20),
            (
                Some(UnalignedSector {
                    aligned_start: 0,
                    offset_into_out_buf: 0,
                    offset_in_sector_buf: 0,
                    len_into_out_buf: 0x20
                }),
                None,
                None
            )
        );

        assert_eq!(
            get_aligned(4, 0x212),
            (
                Some(UnalignedSector {
                    aligned_start: 0,
                    offset_into_out_buf: 0,
                    offset_in_sector_buf: 4,
                    len_into_out_buf: 0x200 - 4
                }),
                None,
                Some(UnalignedSector {
                    aligned_start: 0x200,
                    offset_into_out_buf: 0x200 - 4,
                    offset_in_sector_buf: 0,
                    len_into_out_buf: 0x16
                })
            )
        );

        assert_eq!(
            get_aligned(2, 0xc00),
            (
                Some(UnalignedSector {
                    aligned_start: 0,
                    offset_into_out_buf: 0,
                    offset_in_sector_buf: 2,
                    len_into_out_buf: 0x200 - 2
                }),
                Some(AlignedSector {
                    start: 0x200,
                    len: 0xc00 - 0x200,
                    offset_into_out_buf: 0x200 - 2
                }),
                Some(UnalignedSector {
                    aligned_start: 0xc00,
                    offset_into_out_buf: 0xc00 - 2,
                    offset_in_sector_buf: 0,
                    len_into_out_buf: 2
                })
            )
        )
    }
}
