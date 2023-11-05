pub mod nand;
pub mod nca;
pub mod partitionfs;

pub trait FileSystem {
    type DirEntry;

    fn root(&self) -> Self::DirEntry;
}
