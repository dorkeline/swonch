use super::{IStorage, Storage, SwonchResult};

#[derive(Debug, PartialEq, thiserror_no_std::Error)]
pub enum SubStorageError {
    #[error("attempted to create a too large substorage. parent is {parent_len} large, substorage is {len} bytes at offset {offset} exceeding the parent by {}", parent_len - (offset + len))]
    OutOfBounds {
        parent_len: u64,
        offset: u64,
        len: u64,
    },

    #[error("parent storage needs to declare a length")]
    FailedToGetParentStorageLength,

    #[error("offset overflowed during calculation")]
    OffsetOverflow,
}

pub type SubStorageResult<T> = core::result::Result<T, SubStorageError>;

/// A partial view into an existing [`Storage`]. Useful for logically splitting storages when
/// dealing with containers where a new section logically is a new file with offsets relative to its start.
///
/// ```
/// use swonch::prelude::*;
///
/// fn main() -> SwonchResult<()> {
///     let memory = VecStorage::new([0, 1, 2, 3, 4, 5, 6, 7].into());
///     let first_half = memory.clone().split(0, 4)?;
///     let second_half = memory.clone().split(4, 4)?;
///     
///     let mut buf = [0; 4];
///     first_half.read_at(0, &mut buf)?;
///     assert_eq!(buf, [0, 1, 2, 3]);
///
///     second_half.read_at(0, &mut buf)?;
///     assert_eq!(buf, [4, 5, 6, 7]);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct SubStorage {
    parent: Storage,
    offset: u64,
    len: u64,
}

impl SubStorage {
    pub(crate) fn split_from(
        parent: Storage,
        offset: u64,
        len: u64,
    ) -> Result<Storage, SubStorageError> {
        match parent.length() {
            Ok(parent_len) => {
                if offset + len > parent_len {
                    return Err(SubStorageError::OutOfBounds {
                        parent_len,
                        offset,
                        len,
                    });
                }
            }
            Err(_e) => return Err(SubStorageError::FailedToGetParentStorageLength),
        };

        Ok(Self {
            parent,
            offset,
            len,
        }
        .into_storage())
    }
}

impl IStorage for SubStorage {
    fn read_at(&self, offset: u64, mut buf: &mut [u8]) -> SwonchResult<u64> {
        let buf_len = buf.len();
        let available_len = self.len.saturating_sub(offset);
        buf = &mut buf[..core::cmp::min(available_len as usize, buf_len)];

        if buf.is_empty() {
            return Ok(0);
        }

        self.parent.read_at(
            self.offset
                .checked_add(offset)
                .ok_or(SubStorageError::OffsetOverflow)?,
            buf,
        )
    }

    fn write_at(&self, offset: u64, mut data: &[u8]) -> SwonchResult<u64> {
        let avail_len = self.len.saturating_sub(offset);
        data = &data[..core::cmp::min(avail_len as usize, data.len())];

        if data.is_empty() {
            return Ok(0);
        }

        self.parent.write_at(
            self.offset
                .checked_add(offset)
                .ok_or(SubStorageError::OffsetOverflow)?,
            data,
        )
    }

    fn is_readonly(&self) -> bool {
        self.parent.is_readonly()
    }

    fn length(&self) -> SwonchResult<u64> {
        Ok(self.len)
    }
}

#[cfg(test)]
mod tests {}
