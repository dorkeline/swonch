#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "std", feature(seek_stream_len))]
#![deny(clippy::unwrap_used)]
#![feature(error_in_core)]

extern crate alloc;

pub mod containers;
//pub mod keyset;
pub mod error;
pub mod storage;
pub mod utils;

pub use error::{SwonchError, SwonchResult};

pub mod prelude {
    pub use super::{
        containers::partitionfs::{hfs0, pfs0::Pfs0},
        storage::{IStorage, Storage, VecStorage},
        SwonchError, SwonchResult,
    };
}

//pub use keyset::GLOBAL_KEYS;

#[cfg(feature = "std")]
pub(crate) use parking_lot as sync_impl;

#[cfg(not(feature = "std"))]
pub(crate) use spin as sync_impl;
