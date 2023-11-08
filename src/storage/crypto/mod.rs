pub mod aes_raw;
mod block_buffer;

pub use block_buffer::BlockBufferStorage;

/// A buffered and self aligning wrapper storage for AES128 in XTS mode.
pub type AesXtsStorage = BlockBufferStorage<aes_raw::xts::AesXtsStorage, 0x200>;

/// A buffered and self aligning wrapper storage for AES128 in XTS mode with Nintendo's custom tweak.
pub type AesXtsnStorage = BlockBufferStorage<aes_raw::xts::AesXtsnStorage, 0x200>;
