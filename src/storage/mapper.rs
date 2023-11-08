use super::Storage;

pub trait FromStorage: Sized {
    type Args;
    type Output;

    fn from_storage(parent: Storage, args: Self::Args) -> Self::Output;
}
