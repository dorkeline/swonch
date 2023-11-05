use super::{Storage, SwonchResult};

pub trait FromStorage: Sized {
    type Args;

    fn from_storage(parent: Storage, args: Self::Args) -> SwonchResult<Self>;
}
