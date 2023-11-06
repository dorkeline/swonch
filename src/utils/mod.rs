use core::num::ParseIntError;
use core::fmt;
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

#[derive(thiserror_no_std::Error, Debug)]
pub enum ParseKeyError {
    #[error("keystr did not match the size requested")]
    LengthMismatch {
        requested_key_len: usize,
        actual_key_len: usize,
    },

    #[error("failed parsing an int from the str")]
    ParseIntError(#[from] ParseIntError),
}

pub fn hex_str_to_array<const N: usize>(s: &str) -> Result<[u8; N], ParseKeyError> {
    let mut buf = [0; N];

    if (s.len() / 2) != buf.len() {
        return Err(ParseKeyError::LengthMismatch {
            requested_key_len: N,
            actual_key_len: s.len() / 2,
        });
    }

    for (buf_idx, str_idx) in (0..s.len()).step_by(2).enumerate() {
        buf[buf_idx] = u8::from_str_radix(&s[str_idx..][..2], 16)?;
    }

    Ok(buf)
}

pub(crate) fn dbg_hexdump(mut writer: impl binrw::io::Write, buf: &[u8]) {
    let row_size = 0x20;
    for (idx, row) in buf.chunks(row_size).enumerate() {
        write!(&mut writer, "{:08x}: ", idx * row_size).unwrap();
        for (b_idx, b) in row.into_iter().enumerate() {
            write!(&mut writer, "{b:02x}").unwrap();
            if b_idx != 0 && b_idx.saturating_sub(1) % 2 == 0 {
                write!(&mut writer, " ").unwrap();
            }
        }
        write!(&mut writer, " | ").unwrap();
        for b in row.into_iter() {
            let out = match b {
                0x20..=0x7D => *b as char,
                _ => '.',
            };

            write!(&mut writer, "{}", out).unwrap();
        }
        write!(&mut writer, "\n").unwrap();
    }
    write!(&mut writer, "\n").unwrap();
}

pub fn aes_xtsn_tweak(mut sector: u128) -> [u8; 0x10] {
    let mut tweak = [0; 0x10];
    for b in tweak.iter_mut().rev() {
        *b = (sector & 0xff) as u8;
        sector >>= 8;
    }
    tweak
}

#[binrw::binrw]
#[derive(Clone)]
pub struct HexArray<const N: usize>(pub [u8; N]);

impl<const N: usize> fmt::Debug for HexArray<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        <Self as fmt::Display>::fmt(self, f)?;
        write!(f, "]")
    }
}

impl<const N: usize> fmt::Display for HexArray<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0 {
            write!(f, "{c:02x}")?;
        }
        Ok(())
    }
}
