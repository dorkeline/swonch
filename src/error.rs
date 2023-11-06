#[derive(Debug, thiserror_no_std::Error)]
pub enum SwonchError {
    #[error("binrw error")]
    BinRwError(#[from] binrw::Error),

    #[error("IO error")]
    IoError(#[from] binrw::io::Error),

    #[error("tried to write to a readonly storage")]
    StorageIsReadOnly,

    #[error("error with an nca")]
    Nca(#[from] crate::containers::nca::NcaError),

    #[error("substorage error")]
    SubStorage(#[from] crate::storage::substorage::SubStorageError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// thiserror_no_std does not impl core::error::Error which is behind an nightly feature flag
#[cfg(not(feature = "std"))]
impl core::error::Error for SwonchError {}

impl From<SwonchError> for binrw::io::Error {
    fn from(value: SwonchError) -> Self {
        crate::utils::other_io_error(value)
    }
}

pub type SwonchResult<T> = core::result::Result<T, SwonchError>;
