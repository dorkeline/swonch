#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod containers;
pub mod storage;
pub mod utils;

pub mod prelude {
    pub use super::{
        containers::partitionfs::{hfs0, pfs0::Pfs0},
        storage::Storage,
    };
}
