#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "std", feature(seek_stream_len))]
#![deny(clippy::unwrap_used)]
#![feature(error_in_core, iter_array_chunks)]

#[macro_use]
extern crate alloc;

pub mod common;
pub mod containers;
pub mod error;
pub mod keyset;
pub mod storage;
pub mod utils;

pub use error::{SwonchError, SwonchResult};

pub(crate) use binrw::io;

pub mod prelude {
    pub use super::{
        containers::{
            nca::Nca,
            partitionfs::{hfs0, pfs0::Pfs0},
        },
        storage::{IStorage, Storage, VecStorage},
        SwonchError, SwonchResult,
    };
}

//pub use keyset::KEYS;

#[cfg(feature = "std")]
pub(crate) use parking_lot as sync_impl;

#[cfg(not(feature = "std"))]
pub(crate) use spin as sync_impl;
