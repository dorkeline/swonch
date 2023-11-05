pub mod string_table;

pub(crate) mod sealed {
    pub trait Sealed {}
}

pub(crate) fn other_io_error(
    e: impl core::error::Error + Send + Sync + 'static,
) -> binrw::io::Error {
    use binrw::io::*;

    Error::new(ErrorKind::Other, e)
}
