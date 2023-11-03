pub mod pfs0;

pub trait FileSystem {
    type DirEntry;

    fn root(&self) -> Self::DirEntry;
}
