#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::unwrap_used)]

extern crate alloc;

pub mod containers;
pub mod storage;
pub mod utils;

pub mod prelude {
    pub use super::{
        containers::partitionfs::{hfs0, pfs0::Pfs0},
        storage::Storage,
        SwonchError, SwonchResult,
    };
}

#[cfg(feature = "std")]
pub(crate) use parking_lot as sync_impl;

#[cfg(not(feature = "std"))]
pub(crate) use spin as sync_impl;

#[derive(Debug, thiserror_no_std::Error)]
pub enum SwonchError {
    #[error("binrw error")]
    BinRwError(#[from] binrw::Error),

    #[error("IO error")]
    IoError(#[from] binrw::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type SwonchResult<T> = core::result::Result<T, SwonchError>;
